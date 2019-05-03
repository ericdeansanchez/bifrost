# Bifrost Journal

#### Thursday May 2, 2019
I think it still makes sense to have a global container. However, I don't think
it makes sense to have a global config file. Now that there are workspaces,
users can initialize a `WorkSpace` in their current directory. In this current
working directory it makes sense to have a local config file––much like a `.git`
file.

If this change is made then there will need to be safeguards to prevent 
performing bifrost operations in non-bifrost-initialized directories.

I think `utils.rs` needs some work.

##### Concerns: Owners and Borrowers

###### ./src/config.rs

* ~~fn parse(path: &Option<PathBuf>, config_map: &mut HashMap<String
bool>)~~

###### ./src/utils.rs

**[OK: WorkingDir should own its fields]**
* 18:    `pub root: PathBuf`
* 19:    `pub paths: Vec<PathBuf>`
* 20:    `pub ignore_list: HashSet<PathBuf>`

```rust
pub struct WorkingDir {
    pub root: PathBuf,
    pub paths: Vec<PathBuf>,
    pub ignore_list: HashSet<PathBuf>,
}
```

**[May warrant refactoring]**

* 24:    `pub fn init(cwd: Option<PathBuf>, ignore_list: Vec<PathBuf>)`
* 51:    `fn stash(path: PathBuf, content: &mut WorkingDir)`
* 57:    `fn ignore(&mut self, paths: Vec<PathBuf>)`
* 67:    `pub fn walk_dirs(dir: &PathBuf, visit: &VisitorMut, content: &mut`
* 87:    `pub fn path_builder(prefix: Option<PathBuf>, suffix: &str) -> 
Option<PathBuf>`