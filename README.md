SpOnGiFy
========

A program to make text alternate between upper- and lower-case characters.

![Useful](https://raw.githubusercontent.com/tgockel/spongify/trunk/useful.jpg)

Usage
-----

```sh
$> spongify "your text here"
YoUr tExT HeRe
$> spongify --style "LiKe ThIs" "your text here"
YoUr TeXt HeRe
$> spongify --style "rAnDomly" "your text here"
YouR TeXt HERe
```

Copy to the clipboard with `-c`:

```sh
$> spongify -c "now you can paste sane-cased text anywhere"
```

Read from stdin:

```sh
$> spongify -
hello
HeLlO
is it me you're looking for?
iS It mE YoU'Re lOoKiNg fOr?
```

Offer a SpOnGeCaSe service with netcat:

```sh
mkfifo out
trap "rm -f out" EXIT
while true; do
  cat out | nc -l 24124 > >(
    spongify - > out
  )
done
```

For users:

```sh
nc localhost 24124
Spongecase as a service
SpOnGeCaSe aS A SeRvIcE
```
