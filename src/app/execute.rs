pub trait TryExecute<App> {
    fn try_execute(&self, command: &Vec<&str>, app: &mut App, db: &mut crate::DataBase) -> Result<bool, String>;
}

pub enum ArgPattern {
    Word(regex::Regex),
    Variable(regex::Regex),
}

pub struct CommandExecution<App> {
    pub pattern: Vec<ArgPattern>,
    pub execute: fn(&mut App, Vec<&str>, &mut crate::DataBase) -> Result<(), String>,
}

impl<App> TryExecute<App> for CommandExecution<App> {
    fn try_execute(&self, command: &Vec<&str>, app: &mut App, db: &mut crate::DataBase) -> Result<bool, String> {
        let will_execute = command.len() == self.pattern.len() && (0..command.len()).map(|i| 
            match self.pattern[i] {
                ArgPattern::Word(ref r) => r.is_match(command[i]),
                ArgPattern::Variable(ref r) => r.is_match(command[i])
            }
        ).fold(true, |x, y| x && y);
        if !will_execute { return Ok(false) }
        let args = command.iter().enumerate().filter_map(|(i, x)| matches!(self.pattern[i], ArgPattern::Variable(_)).then_some(*x));
        (self.execute)(app, args.collect(), db).map(|()| true)
    }
}

impl<App> TryExecute<App> for Vec<CommandExecution<App>> {
    fn try_execute(&self, command: &Vec<&str>, app: &mut App, db: &mut crate::DataBase) -> Result<bool, String> {
        for executors in self.iter() {
            match executors.try_execute(command, app, db) {
                Ok(false) => continue,
                Ok(true) => return Ok(true),
                Err(e) => return Err(e),
            }
        }
        Ok(false)
    }
}

#[macro_export]
macro_rules! x_decl {
    ($($x:ident $y:literal, )* |$z0: ident, $z1: ident, $z2: ident| $body: tt) => {
        crate::app::execute::CommandExecution {
            pattern: vec![$(crate::app::execute::x_decl!{$x $y}, )*],
            execute: |$z0, $z1, $z2| $body,
        }
    };
    ($(($($x:tt)*))*) => {
        vec![ $(crate::app::execute::x_decl!{$($x)*}, )* ]
    };
    (w $x:literal) => {
        crate::app::execute::ArgPattern::Word(regex::Regex::new($x).unwrap())
    };
    (v $x:literal) => {
        crate::app::execute::ArgPattern::Variable(regex::Regex::new($x).unwrap())
    };
}

pub(crate) use x_decl;