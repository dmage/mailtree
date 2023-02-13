use clap::Parser;
use maildir::Maildir;
use mailparse::ParsedMail;
use std::collections::HashMap;

#[derive(Parser)]
struct Args {
    maildir: String,
}

struct Mail {
    id: String,
    from: String,
    subject: String,
    parent_id: String,
}

fn convert_mail(m: &ParsedMail) -> Mail {
    let mut id = String::from("");
    let mut from = String::from("");
    let mut subject = String::from("");
    let mut references = String::from("");
    m.headers.iter().for_each(|h| {
        match h.get_key().to_lowercase().as_ref() {
            "message-id" => id = h.get_value(),
            "from" => from = h.get_value(),
            "subject" => subject = h.get_value(),
            "references" => references = h.get_value(),
            _ => (),
        }
    });
    let parent_id = String::from(references.split_whitespace().last().unwrap_or(""));
    Mail {
        id,
        from,
        subject,
        parent_id,
    }
}

fn print_mails(mails: &Vec<Mail>, parent_id: &String, indent: &str) {
    let next_indent = indent.to_owned() + "  ";
    for mail in mails {
        if mail.parent_id == *parent_id {
            let from = mail.from.chars().take(50).collect::<String>();
            println!("{:<50} {} {}", from, indent, mail.subject);
            print_mails(mails, &mail.id, &next_indent);
        }
    }
}

fn main() {
    let args = Args::parse();
    let maildir = Maildir::from(args.maildir);
    let mut mails: Vec<Mail> = Vec::new();
    for entry in maildir.list_cur() {
        let mut entry = entry.unwrap();
        let parsed = entry.parsed().unwrap();
        let mail = convert_mail(&parsed);
        mails.push(mail);
    }
    let by_id: HashMap<String, u32> = mails.iter().map(|m| (m.id.clone(), 1)).collect();
    for mail in &mut mails {
        if mail.parent_id != "" {
            if !by_id.contains_key(&mail.parent_id) {
                mail.parent_id = String::from("");
            }
        }
    }
    print_mails(&mails, &String::from(""), "");
}
