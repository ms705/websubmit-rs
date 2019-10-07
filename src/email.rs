use lettre::{EmailAddress, Envelope, SendableEmail, SmtpClient, Transport};

pub(crate) fn send(
    recipients: Vec<String>,
    subject: String,
    text: String,
) -> Result<lettre::smtp::response::Response, lettre::smtp::error::Error> {
    let email = SendableEmail::new(
        Envelope::new(
            Some(EmailAddress::new("malte@csci2390-submit.cs.brown.edu".to_string()).unwrap()),
            recipients
                .into_iter()
                .map(|r| EmailAddress::new(r).unwrap())
                .collect(),
        )
        .unwrap(),
        subject.to_string(),
        text.to_string().into_bytes(),
    );

    // Open a local connection on port 25
    let mut mailer = SmtpClient::new_unencrypted_localhost().unwrap().transport();
    // Send the email
    let result = mailer.send(email);

    result
}
