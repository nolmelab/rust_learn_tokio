# 환경 준비

* 러스트 설치 (rustup)
* vscode 설치
  * rust analyzer extension 설치
  * codelldb 확장 설치
* cargo new sample로 동작확인&#x20;
  * vscode에서 터미널 열고 cargo build, cargo run&#x20;
* tokio github clone&#x20;

tokio 복제한 폴더에서 vscode를 열고 직접 작업을 해도 좋습니다. 
커밋하지 않을 예정이라면 그렇습니다.

작업한 내용을 보관하고 싶다면 github에 리포를 하나 만들어서 진행하면 좋을 듯 합니다 . 
필요한 외부 패키지들은 tokio 복제 리포의 examples/Cargo.toml의 \[dev-dependencies] 참고해서 설정하면 됩니다.

저는 tokio 복제 리포 내에서 분석 위주로 먼저 진행하고 테스트 필요한 내용은 
example을 추가해서 작업하려고 합니다. 남겨야 할 내용은 깃북에 정리합니다.

