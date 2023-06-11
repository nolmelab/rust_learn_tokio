# library source path 

디버깅 중에 경로가 빌드 경로로 나오기 때문에 소스를 찾을 수 없다. 보통 디버거에 소스 위치를 알려주면 된다. lldb는 어떻게 알려줘야 하는가? 

library의 소스 디버깅이 가능하면 거의 완전하게 코드를 이해할 수 있다. 라이브러리 코드를 읽고 배우고 이해할 수 있는 러스트의 경우는 더 중요하다. 


https://github.com/vadimcn/codelldb/blob/master/MANUAL.md

여기 매뉴얼에 sourceMap 설정이 나온다. 

\rustc\90c541806f23a127002de5b4038be731ba1458ca\library
C:\Users\keedo\.rustup\toolchains\stable-x86_64-pc-windows-msvc\lib\rustlib\src\rust\library

위와 같이 매핑하면 될 것 같다. 

vscode의 settings.json에 아래 항목을 추가하면 매우 잘 된다.

```json
    "lldb.launch.sourceMap": {
        "\\rustc\\90c541806f23a127002de5b4038be731ba1458ca\\": "C:\\Users\\keedo\\.rustup\\toolchains\\stable-x86_64-pc-windows-msvc\\lib\\rustlib\\src\\rust\\"
   }
```

만세~~



