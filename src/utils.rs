use kerberos_asn1::PrincipalName;
use kerberos_constants::principal_names;

pub fn username_to_principal_name(username: String) -> PrincipalName {
    return PrincipalName {
        name_type: principal_names::NT_PRINCIPAL,
        name_string: vec![username],
    };
}

