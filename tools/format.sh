#!/usr/bin/env bash

if [[ 0 -eq `echo $0 | grep -c '^/'` ]]; then
    # relative path
    EXEC_PATH=$(dirname "`pwd`/$0")
else
    # absolute path
    EXEC_PATH=$(dirname "$0")
fi

cd $EXEC_PATH || exit 1

export LC_ALL=en_US.UTF-8 # perl

for file in `find ../src -type f | grep -iEv 'makefile|*.so'`; do
    perl -p -i -e 's/\t/    /g' $file
    perl -p -i -e 's/ +$//g' $file
done

for file in `find ../tests -type f`; do
    perl -p -i -e 's/\t/    /g' $file
    perl -p -i -e 's/ +$//g' $file
done

for file in `find ../tools -type f`; do
    perl -p -i -e 's/\t/    /g' $file
    perl -p -i -e 's/ +$//g' $file
done

cargo fmt --all
