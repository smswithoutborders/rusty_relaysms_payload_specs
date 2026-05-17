use relaysms_specs;
use relaysms_specs::email::Email;

#[test]
fn test_email_init() {
    let to  = "example@gmail.com"; //2
    let body = "Here is some heavy Lorem Ipsum shit"; //4
    let subject = "More things"; //7
    let from_id: u8 = 63; // 1
    let email = Email::new(
        to,
        body,
        subject,
        &from_id
    ).unwrap();

    let serialized = email.serialize();
    let deserialized = Email::deserialize(serialized.as_slice()).unwrap();

    assert_eq!(email, deserialized);

    assert_eq!((to.len() + body.len() + subject.len() + 1 + 3), serialized.len());
}