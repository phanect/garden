use std::env;
use std::fs::{ self, File };
use std::io::{ self, Write };
use std::path::Path;
use std::process::{ Command, exit };

fn vagrant(subcommand: &str, cwd: &Path) {
  let command: &str = &[ "vagrant", subcommand ].join(" ");

  let output = if cfg!(windows) {
    Command::new("cmd")
      .current_dir(cwd)
      .args(&[ "/C", command ])
      .output()
      .expect("Failed to run Vagrant")
  } else {
    Command::new("sh")
      .current_dir(cwd)
      .args(&[ "-c", command ])
      .output()
      .expect("Failed to run Vagrant")
  };

  io::stdout().write_all(&output.stdout).unwrap();
  io::stderr().write_all(&output.stderr).unwrap();
}

fn create(args: Vec<String>, config_root: String) {
  let mut image = args[0].as_str();
  let vm_id = args[1];
  let vm_root = Path::new(&config_root).join(vm_id);

  image = match image {
    "ubuntu" => "hashicorp/bionic64",
    "ubuntu:18:04" => "hashicorp/bionic64",
    "centos" => "centos/8",
    "centos:8" => "centos/8",
    "centos:7" => "centos/7",
    _ => image
  };

  fs::create_dir_all(&vm_root);
  create_vagrantfile(&vm_root, image);
}

fn up(args: Vec<String>, config_root: String) {
  let vm_id = args[0];
  let vm_root = Path::new(&config_root).join(vm_id);
  fs::create_dir_all(&vm_root);
  // create_vagrantfile(&vm_root); // TODO vagrant イメージだけは変えずに synced_folder だけ変えられるように、設定ファイルを用意する
  vagrant("up", &vm_root);
}

fn create_vagrantfile(vm_root: &Path, image: &str) -> std::io::Result<()> {
  let current_dir: std::path::PathBuf = env::current_dir()?;
  // match env::current_dir() {
  //   Ok(result) => result,
  //   Err(msg) => {
  //     println!("{}", msg);
  //     exit(1);
  //   }
  // };
  let vagrantfile_path = vm_root.join("Vagrantfile");
  let mut vagrantfile = File::create(&vagrantfile_path)?;

  if vagrantfile_path.exists() {
    fs::remove_file(vagrantfile_path);
  }

  write!(vagrantfile, "
    Vagrant.configure('2') do |config|
      config.vm.box = '{}'

      # TODO forward all ports
      config.vm.network 'forwarded_port', guest: 80, host: 80
      config.vm.network 'forwarded_port', guest: 443, host: 443

      config.vm.synced_folder '{}', '/home/vagrant/sync'
    end
  ", image, current_dir.to_str().unwrap())?;

  Ok(())
}

fn main() {
  let mut args: Vec<String> = env::args().collect();
  let config_root = if cfg!(windows) {
    match env::var("APPDATA") {
      Ok(val) => [ val, "ivi".to_string() ].join("\\"),
      Err(_) => {
        println!("APPDATA environmental variable is not set.");
        exit(1);
      }
    }
  } else {
    match env::var("HOME") {
      Ok(val) => [ val, ".config/ivi".to_string() ].join("/"),
      Err(_) => {
        println!("HOME environmental variable is not set.");
        exit(1);
      }
    }
  };

  fs::create_dir_all(&config_root);

  args.remove(0); // Remove command name
  let subcommand = args.remove(0); // Remove subcommand name

  match subcommand.as_str() {
    "create" => {
      create(args, config_root);
    },
    "up" => {
      up(args, config_root);
    },
    _ => {
      println!("Unsupported sub-command \"{}\"", subcommand);
    }
  }
}
