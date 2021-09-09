
use r2pipe::{R2Pipe, R2PipeSpawnOptions};
use serde::{Deserialize, Serialize};
use std::u64;
use std::u8;
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, PartialEq)]
pub enum Endian {
    Little,
    Big,
    Mixed,
    Unknown
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallingConvention {
    pub ret:  String,
    pub args: Vec<String>
}

impl Endian {
    pub fn from_string(end: &str) -> Endian {
        match end {
            "little" => Endian::Little,
            "big" => Endian::Big,
            "mixed" => Endian::Mixed,
            _ => Endian::Unknown
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Instruction {
    pub offset: u64,
    pub size: u64,

    #[serde(default="invalid")]
    pub opcode: String,

    #[serde(default="invalid")]
    pub disasm: String,

    #[serde(default="blank")]
    pub esil: String,

    pub bytes: String,

    #[serde(default="invalid")]
    pub r#type: String,

    #[serde(default="zero")]
    pub type_num: i64,

    #[serde(default="zero")]
    pub jump: i64,

    #[serde(default="zero")]
    pub fail: i64  
}

fn invalid() -> String {
    "invalid".to_string()
}

fn blank() -> String {
    "".to_string()
}

fn zero() -> i64 {
    0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub name:  String,
    pub size:  u64,
    pub vsize: u64,
    pub perm:  String,
    pub paddr: u64,
    pub vaddr: u64
}

#[derive(Debug)]
pub struct Permission {
    pub initialized: bool,
    pub read: bool,
    pub write: bool,
    pub execute: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AliasInfo {
    pub reg: String,
    pub role: u64,
    pub role_str: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterInfo {
    pub name: String,
    pub r#type: u64,
    pub type_str: String,
    pub size: u64,
    pub offset: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegisterInformation {
    pub alias_info: Vec<AliasInfo>,
    pub reg_info: Vec<RegisterInfo>
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoreInfo {
    pub file: String,
    pub size: u64,
    pub mode: String,
    pub format: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinInfo {
    pub arch: String,
    pub bintype: String,
    pub bits: u64,
    pub canary: bool,
    pub endian: String,
    pub os: String,
    pub nx: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Information {
    pub core: CoreInfo,
    pub bin: BinInfo
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Syscall {
    pub name: String,
    pub swi: u64,
    pub num: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CrossRef {
    pub addr: u64,
    pub r#type: String,
    pub at: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VarRef {
    pub base: String,
    pub offset: u64
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Variable {
    pub name: String,
    pub kind: String,
    pub r#type: String,
    pub r#ref: VarRef
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub offset: u64,
    pub name: String,
    pub size: u64,
    pub realsz: u64,
    pub noreturn: bool,
    pub stackframe: u64,
    pub calltype: String,
    pub cost: u64,
    pub cc: u64,
    pub bits: u64,
    pub r#type: String,
    pub nbbs: u64, // number of basic blocks
    pub edges: u64,
    pub ebbs: u64, 
    pub signature: String,
    pub minbound: u64,
    pub maxbound: u64,
    pub callrefs: Vec<CrossRef>,
    pub datarefs: Vec<u64>,
    pub codexrefs: Vec<CrossRef>,
    pub dataxrefs: Vec<CrossRef>,
    pub indegree: u64,
    pub outdegree: u64,
    pub nlocals: u64,
    pub nargs: u64,
    pub bpvars: Vec<Variable>,
    pub spvars: Vec<Variable>,
    pub regvars: Vec<Variable>,
    pub difftype: String
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasicBlock {
    pub opaddr: u64,
    pub addr: u64,
    pub size: u64,
    pub inputs: u64,
    pub outputs: u64,
    pub ninstr: u64,
    pub traced: bool,

    #[serde(default="zero")]
    pub jump: i64,

    #[serde(default="zero")]
    pub fail: i64  
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub flagname: String,
    pub realname: String,
    pub ordinal: usize,
    pub bind: String,
    pub size: usize,
    pub r#type: String,
    pub vaddr: u64,
    pub paddr: u64,
    pub is_imported: bool
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub ordinal: usize,
    pub bind: String,
    pub r#type: String,
    pub name: String,
    pub plt: u64
}

pub type R2Result<T> = Result<T, String>;
pub fn r2_result<T, E>(result: Result<T, E>) -> R2Result<T> {
    if let Ok(res) = result {
        Ok(res)
    } else {
        Err("Deserialization error".to_owned())
    }
}

pub fn hex_encode(data: &[u8]) -> String {
    data.iter()
        .map(|d| {format!("{:02x}", *d)})
        .collect::<Vec<_>>()
        .join("")
}

pub fn hex_decode(data: &str) -> Vec<u8> {
    let mut result = Vec::with_capacity(data.len());
    for i in 0..data.len()/2 {
        result.push(u8::from_str_radix(&data[2*i..2*i+2], 16).unwrap());
    }
    result
}

// #[derive(DerefMut)]
#[derive(Clone)]
pub struct R2Api {
    pub r2p: Arc<Mutex<R2Pipe>>,
    //pub instructions: HashMap<u64, Instruction>,
    //pub permissions: HashMap<u64, Permission>,
    pub info: Option<Information>
}

impl R2Api {
    pub fn new(filename: Option<String>, opts: Option<Vec<&'static str>>) -> R2Api {
        let options = if let Some(o) = &opts {
            Some(R2PipeSpawnOptions { 
                exepath: "r2".to_owned(), 
                args: o.clone()
            })
        } else {
            None
        };

        let r2pipe = match (filename, opts) {
            (None, None) => R2Pipe::open(),
            (Some(name), _) => R2Pipe::spawn(name, options),
            _ => Err("cannot have options for non-spawed")
        };

        let mut r2api = R2Api {
            r2p: Arc::new(Mutex::new(r2pipe.unwrap())),
            //instructions: HashMap::new(),
            //permissions: HashMap::new(),
            info: None
        };
    
        let _r = r2api.get_info();
        r2api
    }

    pub fn cmd(&mut self, cmd: &str) -> R2Result<String> {
        self.r2p.lock().unwrap().cmd(cmd)
    }

    pub fn get_info(&mut self) -> R2Result<Information> {
        if self.info.is_none() {
            let json = self.cmd("ij")?;
            self.info = serde_json::from_str(json.as_str()).unwrap()
        }
        Ok(self.info.as_ref().unwrap().clone())
    }

    pub fn get_registers(&mut self) -> R2Result<RegisterInformation> {
        let json = self.cmd("aerpj")?;
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn get_cc(&mut self, pc: u64) -> R2Result<CallingConvention> {
        let json = self.cmd(format!("af @ {}; afcrj @ {}", pc, pc).as_str())?;
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn get_syscall_cc(&mut self, pc: u64) -> R2Result<CallingConvention> {
        let bin = self.info.as_ref().unwrap().bin.clone();
        // this sucks, need a central place for arch shit
        if bin.arch == "x86" && bin.bits == 32 {
            Ok(CallingConvention {
                args: vec!(
                    "ebx".to_string(), 
                    "ecx".to_string(), 
                    "edx".to_string(), 
                    "esi".to_string(), 
                    "edi".to_string(), 
                    "ebp".to_string()
                ),
                ret: "eax".to_string()
            })
        } else {
            self.get_cc(pc)
        }
    }

    pub fn get_segments(&mut self) -> R2Result<Vec<Segment>> {
        let json = self.cmd("iSj")?;
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn analyze(&mut self, n: usize) -> R2Result<String> { 
        // n = 14 automatically wins flareon
        self.cmd("a".repeat(n).as_str())
    }

    pub fn get_function_info(&mut self, addr: u64) -> R2Result<FunctionInfo> {
        let json = self.cmd(format!("afij @ {}", addr).as_str())?;
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn get_functions(&mut self) -> R2Result<Vec<FunctionInfo>> {
        let json = self.cmd("aflj")?;
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn get_blocks(&mut self, addr: u64) -> R2Result<Vec<BasicBlock>> {
        let cmd = format!("af @ {}; afbj @ {}", addr, addr);
        let json = self.cmd(cmd.as_str())?;
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn get_ret(&mut self) -> R2Result<String> {
        // simple as that?
        let ret = self.cmd("pae ret")?;
        Ok(ret[0..ret.len()-1].to_owned())
    }

    pub fn get_register_value(&mut self, reg: &str) -> R2Result<u64> {
        let cmd = format!("aer {}", reg);
        let val = self.cmd(cmd.as_str())?;
        // println!("val: {}", val);
        Ok(u64::from_str_radix(&val[2..val.len()-1], 16).unwrap())
    }

    pub fn set_register_value(&mut self, reg: &str, value: u64) {
        let cmd = format!("aer {}={}", reg, value);
        let _r = self.cmd(cmd.as_str());
    }

    pub fn get_syscall_str(&mut self, sys_num: u64) -> R2Result<String> {
        let cmd = format!("asl {}", sys_num);
        let ret = self.cmd(cmd.as_str())?;
        Ok(ret[0..ret.len()-1].to_owned())
    }

    pub fn get_syscall_num(&mut self, sys_str: &str) -> R2Result<u64> {
        let cmd = format!("asl {}", sys_str);
        let ret = self.cmd(cmd.as_str())?;
        Ok((&ret[0..ret.len()-1]).parse::<u64>().unwrap())
    }
    
    pub fn get_syscalls(&mut self) -> R2Result<Vec<Syscall>> {
        let json = self.cmd("asj")?;
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn seek(&mut self, addr: u64) {
        let _r = self.cmd(format!("s {}", addr).as_str());
    }

    pub fn breakpoint(&mut self, addr: u64) {
        let _r = self.cmd(format!("db {}", addr).as_str());
    }

    pub fn cont(&mut self) {
        let _r = self.cmd("dc");
    }

    pub fn init_vm(&mut self) {
        let _r = self.cmd("aei; aeim");
    }

    pub fn init_entry(&mut self, args: &[String], vars: &[String]) {
        let argc = args.len();
        let argv = args.join(" ");
        let env = vars.join(" ");
        self.init_vm();
        // this is very weird but this is how it works
        let _r = self.cmd(format!(".aeis {} {} {} @ SP", argc, argv, env).as_str());
    }

    pub fn get_symbols(&mut self) -> R2Result<Vec<Symbol>> {
        let json = self.cmd("isj")?;
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn get_imports(&mut self) -> R2Result<Vec<Import>> {
        let json = self.cmd("iij")?;
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn disassemble(&mut self, addr: u64, num: usize) -> R2Result<Vec<Instruction>> {
        let cmd = format!("pdj {} @ {}", num, addr);
        let json = self.cmd(cmd.as_str())?;
        //println!("json: {}", json);
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn disassemble_bytes(&mut self, data: &[u8]) -> R2Result<String> {
        let cmd = format!("pad {}", hex_encode(data));
        self.cmd(cmd.as_str())
    }

    pub fn assemble(&mut self, instruction: &str) -> R2Result<Vec<u8>> {
        let cmd = format!("pa {}", instruction);
        let hexpairs = self.cmd(cmd.as_str())?;
        Ok(hex_decode(&hexpairs))
    }

    pub fn read(&mut self, addr: u64, length: usize) -> R2Result<Vec<u8>> {
        let cmd = format!("xj {} @ {}", length, addr);
        let json = self.cmd(cmd.as_str())?;
        r2_result(serde_json::from_str(json.as_str()))
    }

    pub fn write(&mut self, addr: u64, data: Vec<u8>) {
        let cmd = format!("wx {} @ {}", hex_encode(&data), addr);
        let _r = self.cmd(cmd.as_str());
    }

    pub fn get_address(&mut self, symbol: &str) -> R2Result<u64> {
        let cmd = format!("?v {}", symbol);
        let val = self.cmd(cmd.as_str())?;
        Ok(u64::from_str_radix(&val[2..val.len()-1], 16).unwrap())
    }

    pub fn clear(&mut self) {
        
    }

    pub fn close(&mut self) {
        self.r2p.lock().unwrap().close();
    }
}
/*
impl Drop for R2Api {
    fn drop(&mut self) {
        self.r2p.lock().unwrap().close()
    }
}
*/