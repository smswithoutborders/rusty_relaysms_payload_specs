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
        Option::from(subject),
        &from_id
    ).unwrap();

    let serialized = email.serialize().unwrap();
    let deserialized = Email::deserialize(serialized.as_slice()).unwrap();

    assert_eq!(email, deserialized);
    assert_eq!((to.len() + body.len() + subject.len() + 1 + 3), serialized.len());

    let email1 = Email::new(
        to,
        body,
        None,
        &from_id
    ).unwrap();

    let serialized = email1.serialize().unwrap();
    let deserialized = Email::deserialize(serialized.as_slice()).unwrap();
    assert_eq!(email1, deserialized);
    assert_eq!((to.len() + body.len() + 1 + 2), serialized.len());

}