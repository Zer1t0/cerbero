mod args;
mod ask;
mod cred_format;
mod error;
mod file;
mod kdc_req_builder;
mod krb_cred_plain;
mod krb_user;
mod senders;
mod transporter;
mod utils;

use crate::error::Result;
use args::{args, Arguments, ArgumentsParser};
use ask::{ask_s4u2proxy, ask_s4u2self, ask_tgs, ask_tgt};
use krb_user::KerberosUser;
use std::net::SocketAddr;
use transporter::new_transporter;
use stderrlog;
use log::error;

fn main() {
    let args = ArgumentsParser::parse(&args().get_matches());

    stderrlog::new()
        .module(module_path!())
        .verbosity(args.verbosity)
        .init()
        .unwrap();

    if let Err(error) = main_inner(args) {
        error!("{}", error);
    }
}

fn main_inner(args: Arguments) -> Result<()> {
    let kdc_ip = match args.kdc_ip {
        Some(ip) => ip,
        None => utils::resolve_host(&args.realm)?,
    };

    let kdc_address = SocketAddr::new(kdc_ip, args.kdc_port);
    let transporter = new_transporter(kdc_address, args.transport_protocol);

    let creds_file = utils::get_ticket_file(
        args.out_file,
        &args.username,
        &args.credential_format,
    );

    let impersonate_user = match args.impersonate_user {
        Some(username) => Some(KerberosUser::new(username, args.realm.clone())),
        None => None,
    };

    let user = KerberosUser::new(args.username, args.realm);

    match args.service {
        Some(service) => match impersonate_user {
            Some(impersonate_user) => {
                return ask_s4u2proxy(
                    user,
                    impersonate_user,
                    service,
                    &creds_file,
                    &*transporter,
                    args.user_key.as_ref(),
                    args.credential_format,
                );
            }
            None => {
                return ask_tgs(
                    user,
                    service,
                    &creds_file,
                    &*transporter,
                    args.user_key.as_ref(),
                    args.credential_format,
                );
            }
        },
        None => match impersonate_user {
            Some(impersonate_user) => {
                return ask_s4u2self(
                    user,
                    impersonate_user,
                    &creds_file,
                    &*transporter,
                    args.user_key.as_ref(),
                    args.credential_format,
                );
            }
            None => match &args.user_key {
                Some(user_key) => {
                    return ask_tgt(
                        &user,
                        user_key,
                        args.preauth,
                        &*transporter,
                        &args.credential_format,
                        &creds_file,
                    );
                }
                None => {
                    return Err("Required credentials to request a TGT")?;
                }
            },
        },
    }
}
