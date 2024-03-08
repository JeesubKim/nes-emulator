# nes-emulator


## Zero page
주로 8비트 마프에서 기억장치의 가장 하위부분의 256bytes 영역. 
다른 기억 장치와 달리 1byte만 가지고 주소 지정이 가능하므로 엑세스 시간이 짧다

이영역을 적절히 쓰면 마치 레지스터를 여러개 사용하는 것 처럼 할 수 있고,

주소가 1바이트만 있으면 되므로 명령어의 길이도 짧아진다는 장점이 있음


## Zero page, X
Indexed zero page
색인 레지스터를 사용한 주소 지정방식
다
실제 주소는 두번째 바이트 색인 레지스터에 더해 줌으로 써 얻을 수 있다

The address to be accessed by an instruction using indexed zero page addressing is calculated by taking the 8 bit zero page address from the instruction and adding the current value of the X register to it.

instruction 명령어에 8bit에서 부터 Zero Page Addr을 가져가게 되며, 거기에 X register의 값을 더한다

## Zero page, Y