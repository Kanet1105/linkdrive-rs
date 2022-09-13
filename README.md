# linkdrive.rs

# Installation

1. 아래 링크에서 64비트 운영체제용 러스트 설치

    https://www.rust-lang.org/tools/install 

2. 설치가 끝난 후 선택한 디렉토리에서

    ```git clone https://github.com/Kanet1105/linkdrive-rs.git```

3. linkdrive-rs 폴더 내로 들어간 후 (cd linkdrive-rs)

    ```cargo build --release```

4. 생성된 target/release 폴더 내에서 Settings.toml 폴더를 복사 
붙여넣거나 또는 Settings toml 파일을 target/release 폴더 내에 
새로 생성한 후 아래 Usage block 의 내용을 복사 붙여넣기.

# Usage

```
[default]
# Keywords list
# 
# 아래와 같은 키워드 리스트
# keyword = ["ai", "supply chain"]
# 
# 또는 아래와 같이 multiline 으로 가능
# keyword = [
#     "ai",
#     "supply chain",
#     "distributed system",
# ]
#
# DEFAULT KEYWORD
keyword = ["ai", "supply chain"]

# The recipient.
#
# 한 명이 받아서 전달할 이메일 주소
# 이 주소로 메일이 전달되면 routing 설정을 통해
# 여러 명에게 전달하면 됨.
#
# DEFAULT EMAIL
email = "xxxxxx@gmail.com"

# Weekday
# 
# 이메일을 보낼 요일 설정. 아래의 리스트 중 택 1
# ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
#
# DEFAULT WEEKDAY
weekday = "Sat"

# Time
# 이메일을 보낼 시각 설정
#
# 예시 1: 오전 8시 30분에 이메일 보내도록 설정
# time = "08:30" 
#
# 예시 2: 오후 7시 30분에 이메일 보내도록 설정
# time = "19:30"
#
# DEFAULT TIME
time = "06:30"

# [주의!]
# 절대로 아래 내용을 github 에 업로드하지 마세요.
# 아래 내용은 개인정보가 들어가는 영역입니다.
#
[profile]
id = "SMTP enabled 된 이메일 어카운트"
password = "해당 아이디의 비밀번호"
```
