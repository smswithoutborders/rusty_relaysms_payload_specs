use relaysms_specs;
use relaysms_specs::email::Email;

#[test]
fn test_email_init() {
    let to  = "to";
    let body = "body";
    let subject = "subject";
    let from = "from";
    let email = Email::new(
        to,
        body,
        subject,
        from,
    );
}