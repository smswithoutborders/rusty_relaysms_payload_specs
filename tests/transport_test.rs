use relaysms_specs::Contents;
use relaysms_specs::email::Emails;
use relaysms_specs::transport::Transport;

#[test]
fn transport_test() {
    let to  = "example@gmail.com"; //2
    let body = "Here is some heavy Lorem Ipsum shit"; //4
    let subject = "More things"; //7
    let from_id: u8 = 7; // 1
    let email = Emails::new(
        to,
        body,
        Option::from(subject),
        &from_id
    ).unwrap();

    let i_did = true;
    let version: u8 = 15;
    let encryption_id: u8 = 7;
    let key_id: u8 = 255;
    let device_id= Option::from(rand::random::<[u8; 16]>().to_vec());

    let payload_content: Box<dyn Contents> = Box::new(email);
    let transport = Transport::new(
        i_did,
        version,
        encryption_id,
        key_id,
        device_id,
        payload_content
    ).unwrap();

    let serialized = transport.serialize();
    let deserialized = Transport::deserialize(serialized.unwrap().as_slice()).unwrap();

    assert_eq!(transport, deserialized);
}