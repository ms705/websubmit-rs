use lettre::sendmail::SendmailTransport;
use lettre::Transport;
use lettre_email::Email;

pub(crate) fn send(
    recipients: Vec<String>,
    subject: String,
    text: String,
) -> Result<(), lettre::sendmail::error::Error> {
    let email = Email::builder()
        .from("malte@csci2390-submit.cs.brown.edu")
        .to(recipients[0].clone())
        .subject(subject)
        .text(text)
        .build()
        .expect("couldn't construct email");

    let mut mailer = SendmailTransport::new();
    let result = mailer.send(email.into());

    result
}
