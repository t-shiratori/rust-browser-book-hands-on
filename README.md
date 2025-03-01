
## Overview

My hands-on practice for sababook.

- [作って学ぶ］ブラウザのしくみ──HTTP、HTML、CSS、JavaScriptの裏側 WEB+DB PRESS plus](https://amzn.asia/d/3LgJPEA)
- [Wasabi サポートページ](https://lowlayergirls.github.io/wasabi-help/)

Original source code.
- [SaBA (Sample Browser Application)](https://github.com/d0iasm/sababook)


## How to run

### Start wabsabi os
```bash
./run_on_wasabi.sh
```

If it doesn't start, try again after doing the following:
```bash
export DISPLAY=0
```
※ [run_on_wasabi.shを実行してもQEMUが起動しない](https://lowlayergirls.github.io/wasabi-help/#run_on_wasabish%E3%82%92%E5%AE%9F%E8%A1%8C%E3%81%97%E3%81%A6%E3%82%82qemu%E3%81%8C%E8%B5%B7%E5%8B%95%E3%81%97%E3%81%AA%E3%81%84)

<img width="700" alt="Image" src="https://github.com/user-attachments/assets/9d2ac48a-a706-4fd1-a782-02eb91b9b5c4" />

### Start browser

Input "saba"
```bash
saba
```

<img width="700" alt="Image" src="https://github.com/user-attachments/assets/5ac0834a-9165-45d5-8a77-236fbef559f1" />


### Start web server

```bash
python3 -m http.server 8000
```

Enter `http://host.test:8000/test.html` in the address bar and press enter.


<img width="700" alt="Image" src="https://github.com/user-attachments/assets/7387c268-21cf-402c-90a6-a13a92e59542" />


