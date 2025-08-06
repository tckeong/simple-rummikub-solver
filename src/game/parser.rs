use super::tile_command::{TileCommand, TileCommandError};
use crate::game::{Command, Game};
use regex::Regex;

pub(crate) struct Parser {
    pattern: Regex,
    init_pattern: Regex,
}

#[derive(Debug)]
pub(crate) struct CommandCapture<'a> {
    pub cmd: Option<&'a str>,
    pub idx: Option<&'a str>,
    pub args: Option<&'a str>,
    pub tail: Option<&'a str>,
}

impl Parser {
    pub fn new() -> Self {
        let pattern = Regex::new(
            r"(?x)^
        (?P<cmd>[adp])          
        (?P<idx>\d+|d)?     
        \(?                     
        (?P<args>[^)]*)         
        \)?                     
        (?P<tail>               
            [broh]              
            |                   
            \d+               
        )
    $",
        )
        .unwrap();

        let init_pattern = Regex::new(r"(?P<color>[broh])\((?P<number>\d+|[w])\)").unwrap();

        Parser {
            pattern,
            init_pattern,
        }
    }

    /// Parse one command string and return named captures if matched
    pub fn parse<'a>(&self, input: &'a str) -> Option<CommandCapture<'a>> {
        let captures = self.pattern.captures(input)?;

        Some(CommandCapture {
            cmd: captures.name("cmd").map(|m| m.as_str()),
            idx: captures.name("idx").map(|m| m.as_str()),
            args: captures.name("args").map(|m| m.as_str()),
            tail: captures.name("tail").map(|m| m.as_str()),
        })
    }

    pub fn parse_init<'a>(&self, input: &'a str) -> Option<Vec<CommandCapture<'a>>> {
        let captures = self.init_pattern.captures_iter(input);
        let mut commands = Vec::new();

        for capture in captures {
            let color = capture.name("color")?.as_str();
            let number = capture.name("number")?.as_str();

            commands.push(CommandCapture {
                cmd: Some("a"),
                idx: Some("0"),
                args: Some(number),
                tail: Some(color),
            });
        }

        if commands.is_empty() {
            return None;
        } else {
            Some(commands)
        }
    }
}

impl CommandCapture<'_> {
    pub fn as_tile_command(&self) -> Result<TileCommand, TileCommandError> {
        let cmd = match self.cmd {
            Some("a") => Ok(Command::Add),
            Some("p") => Ok(Command::Put),
            Some("d") => Ok(Command::Draw),
            _ => Err(TileCommandError::InvalidCommand),
        }?;

        let idx = match self.idx {
            Some(idx) => {
                let idx = idx
                    .parse::<usize>()
                    .map_err(|_| TileCommandError::InvalidIndex)?;

                Ok(idx)
            }
            None if cmd == Command::Draw => Ok(0),
            _ => Ok(usize::MAX),
        }?;

        let args = self
            .args
            .map(|args| args.to_string().split(',').map(|s| s.to_string()).collect())
            .ok_or(TileCommandError::InvalidArgs)?;

        let tail = self
            .tail
            .map(|tail| tail.to_string())
            .ok_or(TileCommandError::InvalidTail)?;

        Ok(TileCommand {
            cmd,
            idx,
            args,
            tail,
        })
    }
}

pub fn commands_capture_to_tile_commands(
    commands: Vec<CommandCapture<'_>>,
    game: &Game,
) -> Result<Vec<TileCommand>, TileCommandError> {
    let commands = commands
        .iter()
        .map(|cmd| cmd.as_tile_command())
        .collect::<Vec<_>>();

    let mut tile_command_error = TileCommandError::Other("Default error message".to_string());
    let is_error = commands.iter().any(|cmd| match cmd {
        Ok(_) => false,
        Err(e) => {
            tile_command_error = e.clone();
            true
        }
    });

    if is_error {
        return Err(tile_command_error);
    }

    let commands = commands
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    let commands = commands
        .into_iter()
        .map(|cmd| cmd.validate(game))
        .collect::<Vec<_>>();

    let mut tile_command_error = TileCommandError::Other("Default error message".to_string());
    let is_error = commands.iter().any(|cmd| match cmd {
        Ok(_) => false,
        Err(e) => {
            tile_command_error = e.clone();
            true
        }
    });

    if is_error {
        return Err(tile_command_error);
    }

    let commands = commands
        .into_iter()
        .filter_map(Result::ok)
        .collect::<Vec<_>>();

    Ok(commands)
}

pub fn command_capture_to_tile_command(
    command: CommandCapture<'_>,
    game: &Game,
) -> Result<TileCommand, TileCommandError> {
    let command = command.as_tile_command()?;
    let tile_command = command.validate(game)?;

    Ok(tile_command)
}

#[cfg(test)]
mod tests {
    use crate::game::ToTiles;

    use super::*;

    #[test]
    fn test_parse_simple() {
        let p = Parser::new();
        let cmd = p.parse("a0(3)r").expect("should parse");
        assert_eq!(cmd.idx, Some("0"));
        assert_eq!(cmd.cmd, Some("a"));
        assert_eq!(cmd.args, Some("3"));
        assert_eq!(cmd.tail, Some("r"));

        println!("{:?}", cmd.as_tile_command());
    }

    #[test]
    fn test_parse_2() {
        let p = Parser::new();
        let cmd = p.parse("a1(1,2)b").expect("should parse");
        assert_eq!(cmd.idx, Some("1"));
        assert_eq!(cmd.cmd, Some("a"));
        assert_eq!(cmd.args, Some("1,2"));
        assert_eq!(cmd.tail, Some("b"));
        println!("{:?}", cmd.as_tile_command());
    }

    #[test]
    fn test_parse_3() {
        let p = Parser::new();
        let cmd = p.parse("a1(r,b)2").expect("should parse");
        assert_eq!(cmd.idx, Some("1"));
        assert_eq!(cmd.cmd, Some("a"));
        assert_eq!(cmd.args, Some("r,b"));
        assert_eq!(cmd.tail, Some("2"));
        println!("{:?}", cmd.as_tile_command());
    }

    #[test]
    fn test_parse_4() {
        let p = Parser::new();
        let cmd = p.parse("a1(11,12,w)b").expect("should parse");

        assert_eq!(cmd.idx, Some("1"));
        assert_eq!(cmd.cmd, Some("a"));
        assert_eq!(cmd.args, Some("11,12,w"));
        assert_eq!(cmd.tail, Some("b"));
    }

    #[test]
    fn test_parse_5() {
        let p = Parser::new();
        let cmd = p.parse("a1(r,b,w)10").expect("should parse");

        assert_eq!(cmd.idx, Some("1"));
        assert_eq!(cmd.cmd, Some("a"));
        assert_eq!(cmd.args, Some("r,b,w"));
        assert_eq!(cmd.tail, Some("10"));

        println!("{:?}", cmd.as_tile_command());

        println!("{:?}", cmd.as_tile_command().unwrap().to_tiles());
    }

    #[test]
    fn test_parse_draw() {
        let p = Parser::new();
        let cmd = p.parse("d(10)r").expect("should parse");
        assert_eq!(cmd.cmd, Some("d"));
        assert_eq!(cmd.args, Some("10"));
        assert_eq!(cmd.tail, Some("r"));

        let tile_command = cmd
            .as_tile_command()
            .expect("should convert to TileCommand");

        println!("{:?}", cmd.as_tile_command());

        assert!(tile_command.cmd == Command::Draw);
        assert!(tile_command.idx == 0);
        assert!(tile_command.args == vec!["10".to_string()]);
        assert!(tile_command.tail == "r".to_string());
    }

    #[test]
    fn test_parse_draw2() {
        let p = Parser::new();
        let cmd = p.parse("d(10)h").expect("should parse");
        assert_eq!(cmd.cmd, Some("d"));
        assert_eq!(cmd.args, Some("10"));
        assert_eq!(cmd.tail, Some("h"));

        let tile_command = cmd
            .as_tile_command()
            .expect("should convert to TileCommand");
        println!("{:?}", cmd.as_tile_command());
        assert!(tile_command.cmd == Command::Draw);
        assert!(tile_command.idx == 0);
        assert!(tile_command.args == vec!["10".to_string()]);
        assert!(tile_command.tail == "h".to_string());
    }

    #[test]
    fn test_parse_put() {
        let p = Parser::new();
        let cmd = p.parse("p(9,10,11)h").expect("should parse");
        assert_eq!(cmd.cmd, Some("p"));
        assert_eq!(cmd.args, Some("9,10,11"));
        assert_eq!(cmd.tail, Some("h"));
        println!("{:?}", cmd.as_tile_command());
    }

    #[test]
    fn test_parse_init() {
        let p = Parser::new();
        let input = "r(1)b(2)h(3)";
        let commands = p.parse_init(input).expect("should parse init commands");
        let commands = commands
            .iter()
            .map(|cmd| cmd.as_tile_command())
            .collect::<Vec<_>>();
        println!("{:?}", commands);
    }

    #[test]
    fn test_parse_init2() {
        let p = Parser::new();
        let input = "r(1)b(2)h(3)o(w)";
        let commands_init = p.parse_init(input).expect("should parse init commands");
        let commands = commands_init
            .iter()
            .map(|cmd| cmd.as_tile_command())
            .collect::<Vec<_>>();

        println!("{:?}", commands);

        let tiles = commands
            .into_iter()
            .map(|cmd| cmd.unwrap().to_tiles())
            .collect::<Vec<_>>();

        println!("{:?}", tiles);
    }
}
