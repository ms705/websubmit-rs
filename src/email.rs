use lettre::sendmail::SendmailTransport;
use lettre::Transport;
use lettre_email::Email;
use std::fs;
use std::path::Path;

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

pub(crate) fn send_with_attachment(
    sender: String,
    recipient: String,
    subject: String,
    text: String,
    path: String,
) -> Result<(), lettre::sendmail::error::Error> {
    let mut mailer = SendmailTransport::new();

    let path = Path::new(&path);
    let email = Email::builder()
        .from(sender.clone())
        .to(recipient)
        .subject(subject.clone())
        .text(text.clone())
        .attachment_from_file(path, None, &mime::APPLICATION_JSON)
        .unwrap()
        .build()
        .expect("couldn't construct email");
    mailer.send(email.into())?;

    fs::remove_file(path)?;

    Ok(())
}
