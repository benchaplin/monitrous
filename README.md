# monitrous

Monitor a list of URLs by capturing screenshots and comparing them.

## Usage

### `capture`

Create a text file of URLs to monitor, separated by line. For example:

**urls.txt**

```
https://reddit.com/r/news
https://reddit.com/r/gifs
```

Run the binary with options `-i urls.txt -o my-screenshots` to screenshot the URLs in urls.txt, and put the JPG images in a directory my-screenshots.

```
./bin/monitrous -i urls.txt -o my-screenshots
```
