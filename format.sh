#!/usr/bin/env bash

cargo fmt --all

for x in $(find `pwd` -name "*.rs" -type f); do
	#全部使用空格缩进
	perl -pi -e 's/\t/    /g' $x

	#清除行末的空白
	perl -pi -e 's/ +(?=$)//g' $x
done
