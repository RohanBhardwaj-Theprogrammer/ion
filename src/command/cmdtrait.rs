
use crate::cmd_parser::CmdType;

trait CmdTrait {
    fn execute(&self, commands:&CmdType) -> Result<String>;
}