# sneaky-png
Naïve PNG steganography in Rust.


## How does it work?
For a chosen <i>N</i> = bitmask size (default 3), each channel of each RGBA pixel
will have its less significant <i>N</i> bits stripped away and replaced by <i>N</i>
bits of data.

Depending on the nature of the base images, bitmask sizes of either 3 or 4 will
work best, resulting in very discreet artifacts and a storage efficiency of
~38 or ~50%, respectively.

Data that does not follow a pseudo-random distribution (e.g. ciphered data)
is not recommended, as it will most likely cause visible patterns;


## Bitmask range
<p align="center">
   <i>Demo encoding of text data using mask bitsizes from 1 to 8</i><br><br/>

   <img src="https://raw.githubusercontent.com/g-andrade/sneaky-png/master/range_example/Lenna_1bits.png"
         width="256px" height="256px">&nbsp;&nbsp;
   <img src="https://raw.githubusercontent.com/g-andrade/sneaky-png/master/range_example/Lenna_2bits.png"
         width="256px" height="256px"><br/>

   <img src="https://raw.githubusercontent.com/g-andrade/sneaky-png/master/range_example/Lenna_3bits.png"
         width="256px" height="256px">&nbsp;&nbsp;
   <img src="https://raw.githubusercontent.com/g-andrade/sneaky-png/master/range_example/Lenna_4bits.png"
         width="256px" height="256px"><br/>

   <img src="https://raw.githubusercontent.com/g-andrade/sneaky-png/master/range_example/Lenna_5bits.png"
         width="256px" height="256px">&nbsp;&nbsp;
   <img src="https://raw.githubusercontent.com/g-andrade/sneaky-png/master/range_example/Lenna_6bits.png"
         width="256px" height="256px"><br/>

   <img src="https://raw.githubusercontent.com/g-andrade/sneaky-png/master/range_example/Lenna_7bits.png"
         width="256px" height="256px">&nbsp;&nbsp;
   <img src="https://raw.githubusercontent.com/g-andrade/sneaky-png/master/range_example/Lenna_8bits.png"
         width="256px" height="256px"><br/>
</p>


## Why does it work?
By stripping away the less significant bits, the resulting changes in colour/alpha
will be small when compared to the scale of the more significant bits; and by encoding
data that follows a pseudo-random distribution (which is not, however, part of this
software's scope), we can expect a nice average/stddev for precision changes (over
large enough areas) and therefore to lack any noticetable patterns.


## Encoding
   - Input: new images' output directory;
   - Input: base images' paths;
   - Input: stdin (data before encoding);
   - Output: modified images (into the aforementioned directory.)


## Decoding
   - Input: encoded images directory;
   - Output: stdout (data after decoding)


## Usage
Decoding is default, encoding is explicit.
```
$ ./target/release/sneaky-png --help
Usage: ./target/release/sneaky-png [options] [image1 [image2 ..

Options:
    -h --help           print this help menu
    -e --encode PATH    encode images and put them in PATH
    -b --bitmask_size   size (in bits) of the blending mask
```

## Technical details:
   - Used rust nightly (1.1.0-nightly (7a52835c1 2015-05-16)) due to dependency on
     instable features.
