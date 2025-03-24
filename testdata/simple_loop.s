MOV A, #0

start:
EQ A, #10
CJP end

ADD A, #1
JMP start

end:
ADD FLAGS, #1
