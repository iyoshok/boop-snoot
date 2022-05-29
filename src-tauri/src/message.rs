use std::fmt::Display;

#[derive(Debug, PartialEq)]
pub enum MessageType {
    // usually requests
    CONNECT(String, String), // key, password
    DISCONNECT,
    PING,
    BOOP(String), // partner_key
    AYT(String),  // partner_key

    // usually responses
    HEY,
    NO,
    BYE,
    PONG,
    ERROR(MessageErrorKind),
    ONLINE(String),
    AFK(String)
}

#[derive(Debug, PartialEq)]
pub enum MessageErrorKind {
    NotAvailable,
    MalformedCommand,
    MalformedArguments,
    ProtocolMismatch
}

#[derive(Debug, PartialEq)]
pub enum ParserError {
    UnknownMessageType,
    UnknownArguments
}

impl Display for ParserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParserError::UnknownMessageType => write!(f, "The message type text is invalid."),
            ParserError::UnknownArguments => write!(
                f,
                "The provided arguments are not appropriate for the selected message type."
            )
        }
    }
}

impl Into<MessageErrorKind> for ParserError {
    fn into(self) -> MessageErrorKind {
        match self {
            ParserError::UnknownMessageType => MessageErrorKind::MalformedCommand,
            ParserError::UnknownArguments => MessageErrorKind::MalformedArguments
        }
    }
}

fn connect(args: &Vec<&str>) -> Result<MessageType, ParserError> {
    if args.len() == 2 {
        Ok(MessageType::CONNECT(
            String::from(args[0]),
            String::from(args[1])
        ))
    } else {
        Err(ParserError::UnknownArguments)
    }
}

fn boop(args: &Vec<&str>) -> Result<MessageType, ParserError> {
    if args.len() == 1 {
        Ok(MessageType::BOOP(String::from(args[0])))
    } else {
        Err(ParserError::UnknownArguments)
    }
}

fn ayt(args: &Vec<&str>) -> Result<MessageType, ParserError> {
    if args.len() == 1 {
        Ok(MessageType::AYT(String::from(args[0])))
    } else {
        Err(ParserError::UnknownArguments)
    }
}

fn online(args: &Vec<&str>) -> Result<MessageType, ParserError> {
    if args.len() == 1 {
        Ok(MessageType::ONLINE(String::from(args[0])))
    } else {
        Err(ParserError::UnknownArguments)
    }
}

fn afk(args: &Vec<&str>) -> Result<MessageType, ParserError> {
    if args.len() == 1 {
        Ok(MessageType::AFK(String::from(args[0])))
    } else {
        Err(ParserError::UnknownArguments)
    }
}

fn error(args: &Vec<&str>) -> Result<MessageType, ParserError> {
    if args.len() == 1 {
        match args[0] {
            "NOT_AVAILABLE" => Ok(MessageType::ERROR(MessageErrorKind::NotAvailable)),
            "MALFORMED_COMMAND" => Ok(MessageType::ERROR(MessageErrorKind::MalformedCommand)),
            "MALFORMED_ARGUMENTS" => Ok(MessageType::ERROR(MessageErrorKind::MalformedArguments)),
            "PROTOCOL_MISMATCH" => Ok(MessageType::ERROR(MessageErrorKind::ProtocolMismatch)),
            _ => Err(ParserError::UnknownArguments)
        }
    } else {
        Err(ParserError::UnknownArguments)
    }
}

fn get_message_type_from_text(cmd: &str, args: Vec<&str>) -> Result<MessageType, ParserError> {
    if args.len() == 0 {
        match cmd.to_ascii_uppercase().as_str() {
            "DISCONNECT" => Ok(MessageType::DISCONNECT),
            "PING" => Ok(MessageType::PING),
            "HEY" => Ok(MessageType::HEY),
            "NO" => Ok(MessageType::NO),
            "PONG" => Ok(MessageType::PONG),
            "BYE" => Ok(MessageType::BYE),

            // catch errors
            "CONNECT" => Err(ParserError::UnknownArguments),
            "BOOP" => Err(ParserError::UnknownArguments),
            "AYT" => Err(ParserError::UnknownArguments),
            "ERROR" => Err(ParserError::UnknownArguments),
            "ONLINE" => Err(ParserError::UnknownArguments),
            "AFK" => Err(ParserError::UnknownArguments),
            _ => Err(ParserError::UnknownMessageType)
        }
    } else {
        match cmd.to_ascii_uppercase().as_str() {
            "CONNECT" => connect(&args),
            "BOOP" => boop(&args),
            "AYT" => ayt(&args),
            "ERROR" => error(&args),
            "ONLINE" => online(&args),
            "AFK" => afk(&args),

            // catch errors
            "DISCONNECT" => Err(ParserError::UnknownArguments),
            "PING" => Err(ParserError::UnknownArguments),
            "HEY" => Err(ParserError::UnknownArguments),
            "NO" => Err(ParserError::UnknownArguments),
            "PONG" => Err(ParserError::UnknownArguments),
            "BYE" => Err(ParserError::UnknownArguments),
            _ => Err(ParserError::UnknownMessageType)
        }
    }
}

pub fn parse_message(msg: &String) -> Result<MessageType, ParserError> {
    let mut cmd = msg.clone();
    // remove newline if it's still at the end
    if cmd.ends_with("\n") {
        cmd.remove(cmd.len() - 1); // remove newline char
    }
    cmd = String::from(cmd.trim());
    let mut split: Vec<&str> = cmd.split(" ").collect();
    get_message_type_from_text(split.remove(0), split)
}

pub fn create_message_text(msg_type: MessageType) -> String {
    match msg_type {
        MessageType::CONNECT(key, password) => format!("CONNECT {} {}\n", key, password),
        MessageType::DISCONNECT => String::from("DISCONNECT\n"),
        MessageType::PING => String::from("PING\n"),
        MessageType::BOOP(partner_key) => format!("BOOP {}\n", partner_key),
        MessageType::AYT(partner_key) => format!("AYT {}\n", partner_key),
        MessageType::HEY => String::from("HEY\n"),
        MessageType::NO => String::from("NO\n"),
        MessageType::BYE => String::from("BYE\n"),
        MessageType::PONG => String::from("PONG\n"),
        MessageType::ERROR(err_kind) => format!("ERROR {}\n", error_text(err_kind)),
        MessageType::ONLINE(partner_key) => format!("ONLINE {}\n", partner_key),
        MessageType::AFK(partner_key) => format!("AFK {}\n", partner_key)
    }
}

pub fn error_text(err_kind: MessageErrorKind) -> String {
    let kind_text = match err_kind {
        MessageErrorKind::NotAvailable => "NOT_AVAILABLE",
        MessageErrorKind::MalformedCommand => "MALFORMED_COMMAND",
        MessageErrorKind::MalformedArguments => "MALFORMED_ARGUMENTS",
        MessageErrorKind::ProtocolMismatch => "PROTOCOL_MISMATCH"
    };

    String::from(kind_text)
}

// #############################################################################
// ########## ######################################## TESTS
// ######################################## ####################################
// ###################################################

#[cfg(test)]
mod tests {
    use crate::message::{
        parse_message,
        MessageType,
        ParserError
    };

    #[test]
    fn test_parser_correct() {
        // two values
        let teststring = String::from("CONNECT foo bar\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_ok());
        assert_eq!(
            test_res.unwrap(),
            MessageType::CONNECT(String::from("foo"), String::from("bar"))
        );

        // one value
        let teststring = String::from("BOOP foo\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_ok());
        assert_eq!(test_res.unwrap(), MessageType::BOOP(String::from("foo")));

        // no values
        let teststring = String::from("PING\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_ok());
        assert_eq!(test_res.unwrap(), MessageType::PING);

        // change case
        let teststring = String::from("coNnECt foo bar\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_ok());
        assert_eq!(
            test_res.unwrap(),
            MessageType::CONNECT(String::from("foo"), String::from("bar"))
        );

        // no newline char
        let teststring = String::from("coNnECt foo bar");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_ok());
        assert_eq!(
            test_res.unwrap(),
            MessageType::CONNECT(String::from("foo"), String::from("bar"))
        );
    }

    #[test]
    fn test_parser_incorrect() {
        // invalid command
        let teststring = String::from("DOESNOTEXIST foo bar\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_err());
        assert_eq!(test_res.unwrap_err(), ParserError::UnknownMessageType);

        // missing arguments / 1
        let teststring = String::from("BOOP\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_err());
        assert_eq!(test_res.unwrap_err(), ParserError::UnknownArguments);

        // missing arguments / 2
        let teststring = String::from("CONNECT foo\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_err());
        assert_eq!(test_res.unwrap_err(), ParserError::UnknownArguments);

        // too many arguments / 1
        let teststring = String::from("BOOP foo bar\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_err());
        assert_eq!(test_res.unwrap_err(), ParserError::UnknownArguments);

        // too many arguments / 1.2
        let teststring = String::from("BOOP foo bar bar foo\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_err());
        assert_eq!(test_res.unwrap_err(), ParserError::UnknownArguments);

        // missing arguments / 2
        let teststring = String::from("CONNECT foo bar bar\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_err());
        assert_eq!(test_res.unwrap_err(), ParserError::UnknownArguments);

        // missing arguments / 2.2
        let teststring = String::from("CONNECT foo bar bar foo\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_err());
        assert_eq!(test_res.unwrap_err(), ParserError::UnknownArguments);

        // empty arguments / 1
        let teststring = String::from("BOOP  \n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_err());
        assert_eq!(test_res.unwrap_err(), ParserError::UnknownArguments);

        // empty arguments / 2
        let teststring = String::from("CONNECT   bar\n");
        let test_res = parse_message(&teststring);
        assert!(test_res.is_err());
        assert_eq!(test_res.unwrap_err(), ParserError::UnknownArguments);
    }
}
