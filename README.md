# linkdrive.rs

# Installation

1. 아래 링크에서 64비트 운영체제용 러스트 설치
https://www.rust-lang.org/tools/install 

2. 설치가 끝난 후 선택한 디렉토리에서
git clone https://github.com/Kanet1105/linkdrive-rs.git

3. linkdrive-rs 폴더 내로 들어간 후 (cd linkdrive-rs)
cargo build --release

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
# 또는 아래와 같이도 가능
# keyword = [
# 
# ]
# The recipient.
# 한 명이 받아서 전달할 이메일 주소
email = "받을 이메일 주소"

# Choose a weekday to send an email on.
# ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"]
weekday = "Sat"

# Choose a time to send an email on. 
time = "HH:MM"

[profile]
id = "SMTP enabled 된 이메일 어카운트"
password = "해당 아이디의 비밀번호"
```
