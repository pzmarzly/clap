// Std
use std::io::Write;

// Internal
use app::App;

pub struct FishGen<'a, 'b>(&'b App<'a, 'b>)
where
    'a: 'b;

impl<'a, 'b> FishGen<'a, 'b> {
    pub fn new(app: &'b App<'a, 'b>) -> Self { FishGen(app) }

    pub fn generate_to<W: Write>(&self, buf: &mut W) {
        let command = self.0.bin_name.as_ref().unwrap();

        // function to detect subcommand
        let detect_subcommand_function = r#"function __fish_using_command
    set cmd (commandline -opc)
    if [ (count $cmd) -eq (count $argv) ]
        for i in (seq (count $argv))
            if [ $cmd[$i] != $argv[$i] ]
                return 1
            end
        end
        return 0
    end
    return 1
end

"#.to_string();

        let mut buffer = detect_subcommand_function;
        gen_fish_inner(command, self, &command.to_string(), &mut buffer);
        w!(buf, buffer.as_bytes());
    }
}

// Escape string inside single quotes
fn escape_string(string: &str) -> String { string.replace("\\", "\\\\").replace("'", "\\'") }

fn gen_fish_inner(root_command: &str, comp_gen: &FishGen, parent_cmds: &str, buffer: &mut String) {
    debugln!("FishGen::gen_fish_inner;");
    // example :
    //
    // complete
    //      -c {command}
    //      -d "{description}"
    //      -s {short}
    //      -l {long}
    //      -a "{possible_arguments}"
    //      -r # if require parameter
    //      -f # don't use file completion
    //      -n "__fish_using_command myprog subcmd1" # complete for command "myprog subcmd1"

    let basic_template = format!(
        "complete -c {} -n \"__fish_using_command {}\"",
        root_command, parent_cmds
    );

    for option in opts!(comp_gen.0) {
        let mut template = basic_template.clone();
        if let Some(data) = option.short {
            template.push_str(format!(" -s {}", data).as_str());
        }
        if let Some(data) = option.long {
            template.push_str(format!(" -l {}", data).as_str());
        }
        if let Some(data) = option.help {
            template.push_str(format!(" -d '{}'", escape_string(data)).as_str());
        }
        if let Some(ref data) = option.possible_vals {
            template.push_str(format!(" -r -f -a \"{}\"", data.join(" ")).as_str());
        }
        buffer.push_str(template.as_str());
        buffer.push_str("\n");
    }

    for flag in flags!(comp_gen.0) {
        let mut template = basic_template.clone();
        if let Some(data) = flag.short {
            template.push_str(format!(" -s {}", data).as_str());
        }
        if let Some(data) = flag.long {
            template.push_str(format!(" -l {}", data).as_str());
        }
        if let Some(data) = flag.help {
            template.push_str(format!(" -d '{}'", escape_string(data)).as_str());
        }
        buffer.push_str(template.as_str());
        buffer.push_str("\n");
    }

    for subcommand in subcommands!(comp_gen.0) {
        let mut template = basic_template.clone();
        template.push_str(" -f");
        template.push_str(format!(" -a \"{}\"", &subcommand.name).as_str());
        if let Some(data) = subcommand.about {
            template.push_str(format!(" -d '{}'", escape_string(data)).as_str())
        }
        buffer.push_str(template.as_str());
        buffer.push_str("\n");
    }

    // generate options of subcommands
    for subcommand in subcommands!(comp_gen.0) {
        let sub_comp_gen = FishGen::new(&subcommand);
        // make new "parent_cmds" for different subcommands
        let mut sub_parent_cmds = parent_cmds.to_string();
        if !sub_parent_cmds.is_empty() {
            sub_parent_cmds.push_str(" ");
        }
        sub_parent_cmds.push_str(&subcommand.name);
        gen_fish_inner(root_command, &sub_comp_gen, &sub_parent_cmds, buffer);
    }
}
