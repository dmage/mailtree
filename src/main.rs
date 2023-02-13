use clap::Parser;
use maildir::Maildir;
use mailparse::ParsedMail;
use std::collections::HashMap;

#[derive(Parser)]
struct Args {
    maildir: String,
}

struct Mail {
    fake: bool,
    id: String,
    from: String,
    subject: String,
    parent_id: String,
    references: Vec<String>,
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
        fake: false,
        id,
        from,
        subject,
        parent_id,
        references: references.split_whitespace().map(|s| s.to_string()).collect(),
    }
}

fn print_mails(mails: &Vec<Mail>, parent_id: &String, indent: &str) {
    let next_indent = indent.to_owned() + "  ";
    for mail in mails {
        if mail.parent_id == *parent_id {
            let from = mail.from.chars().take(50).collect::<String>();
            if !mail.fake {
                println!("{:<50} {} {}", from, indent, mail.subject);
            }
            print_mails(mails, &mail.id, if mail.fake { indent } else { &next_indent });
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
    let mut by_id: HashMap<String, u32> = mails.iter().map(|m| (m.id.clone(), 1)).collect();
    for mail in &mut mails {
        for r in &mail.references {
            if !by_id.contains_key(r) {
                by_id.insert(r.clone(), 0);
            }
        }
    }
    for (id, count) in by_id.iter() {
        if *count == 0 {
            mails.push(Mail {
                fake: true,
                id: id.clone(),
                from: String::from(""),
                subject: String::from(""),
                parent_id: String::from(""),
                references: Vec::new(),
            });
        }
    }

    print_mails(&mails, &String::from(""), "");
}
