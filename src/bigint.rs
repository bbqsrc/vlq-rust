use num_bigint::{BigInt, BigUint, ToBigInt, Sign};

impl crate::Vlq for BigUint {
    fn from_reader<R: std::io::Read>(reader: &mut R) -> std::io::Result<Self> {
        let mut stream = vec![];
        let mut buf = [0; 1];

        loop {
            reader.read_exact(&mut buf)?;
            stream.push(buf[0] & 0b0111_1111);

            if (buf[0] & 0b1000_0000) == 0 {
                break;
            }
        }

        Ok(BigUint::from_radix_le(&stream, 0b0111_1111).unwrap())
    }

    fn to_writer<W: std::io::Write>(self, writer: &mut W) -> std::io::Result<()> {
        let mut stream = self.to_radix_le(0b0111_1111);
        stream.iter_mut().for_each(|x| *x |= 0b1000_0000);
        *stream.last_mut().unwrap() &= 0b0111_1111;
        writer.write_all(&stream)
    }
}
