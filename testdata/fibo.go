package main
import (
	"fmt"
)



func main() {
	a:= 0
	b:= 1
	
	for i:=0; i<10; i++ {
		fmt.Println(a)
		c:=a+b
		a=b
		b=c
	}

}