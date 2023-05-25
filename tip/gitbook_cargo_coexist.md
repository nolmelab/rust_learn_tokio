# 깃북과 카고 공존하기 

무언가를 배울 때 문서화와 함께 코딩을 할 수 있으면 좋습니다. 
이럴 경우 vscode에서 rust analyzer가 함께 동작해야 하는데 
깃북 폴더들 때문에 활성화가 되지 않습니다. 

```log
 rust-lang.rust-analyzer, startup: true, activationEvent: 'workspaceContains:*/Cargo.toml,*/rust-project.json'
 ```

*/rust-project.json과 Cargo.toml을 동시에 찾으므로 깃북 폴더를 포함하여 
모든 차상위 폴더에 Cargo.toml이나 rust-project.json 파일이 있으면 
rust-analyzer가 활성화 됩니다. 

