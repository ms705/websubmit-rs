use lettre::sendmail::SendmailTransport;
use lettre::Transport;
use lettre_email::Email;

pub(crate) fn send(
    sender: String,
    recipients: Vec<String>,
    subject: String,
    text: String,
) -> Result<(), lettre::sendmail::error::Error> {
    let mut mailer = SendmailTransport::new();

    for recipient in recipients {
        let email = Email::builder()
            .from(sender.clone())
            .to(recipient)
            .subject(subject.clone())
            .text(text.clone())
            .build()
            .expect("couldn't construct email");
        mailer.send(email.into())?
    }

    Ok(())
}
