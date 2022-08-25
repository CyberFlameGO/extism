use crate::*;

macro_rules! plugin {
    (mut $a:expr) => {
        unsafe { (&mut *$a.plugin) }
    };

    ($a:expr) => {
        unsafe { (&*$a.plugin) }
    };
}

macro_rules! memory {
    (mut $a:expr) => {
        &mut plugin!(mut $a).memory
    };

    ($a:expr) => {
        &plugin!($a).memory
    };
}

pub(crate) fn input_offset(
    caller: Caller<Internal>,
    _input: &[Val],
    output: &mut [Val],
) -> Result<(), Trap> {
    let data: &Internal = caller.data();
    output[0] = Val::I64(data.input_offset as i64);
    Ok(())
}

pub(crate) fn output_set(
    mut caller: Caller<Internal>,
    input: &[Val],
    _output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    data.output_offset = input[0].unwrap_i64() as usize;
    data.output_length = input[1].unwrap_i64() as usize;
    Ok(())
}

pub(crate) fn alloc(
    mut caller: Caller<Internal>,
    input: &[Val],
    output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let offs = memory!(mut data).alloc(input[0].unwrap_i64() as _)?;
    output[0] = Val::I64(offs.offset as i64);

    Ok(())
}

pub(crate) fn free(
    mut caller: Caller<Internal>,
    input: &[Val],
    _output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let offset = input[0].unwrap_i64() as usize;
    memory!(mut data).free(offset);
    Ok(())
}

pub(crate) fn store_u8(
    mut caller: Caller<Internal>,
    input: &[Val],
    _output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let byte = input[1].unwrap_i32() as u8;
    memory!(mut data)
        .store_u8(input[0].unwrap_i64() as usize, byte)
        .map_err(|_| Trap::new("Write error"))?;
    Ok(())
}

pub(crate) fn load_u8(
    mut caller: Caller<Internal>,
    input: &[Val],
    output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let byte = memory!(data)
        .load_u8(input[0].unwrap_i64() as usize)
        .map_err(|_| Trap::new("Read error"))?;
    output[0] = Val::I32(byte as i32);
    Ok(())
}

pub(crate) fn store_u32(
    mut caller: Caller<Internal>,
    input: &[Val],
    _output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let b = input[1].unwrap_i32() as u32;
    memory!(mut data)
        .store_u32(input[0].unwrap_i64() as usize, b)
        .map_err(|_| Trap::new("Write error"))?;
    Ok(())
}

pub(crate) fn load_u32(
    mut caller: Caller<Internal>,
    input: &[Val],
    output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let b = memory!(data)
        .load_u32(input[0].unwrap_i64() as usize)
        .map_err(|_| Trap::new("Read error"))?;
    output[0] = Val::I32(b as i32);
    Ok(())
}

pub(crate) fn store_u64(
    mut caller: Caller<Internal>,
    input: &[Val],
    _output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let b = input[1].unwrap_i64() as u64;
    memory!(mut data)
        .store_u64(input[0].unwrap_i64() as usize, b)
        .map_err(|_| Trap::new("Write error"))?;
    Ok(())
}

pub(crate) fn load_u64(
    mut caller: Caller<Internal>,
    input: &[Val],
    output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let byte = memory!(data)
        .load_u64(input[0].unwrap_i64() as usize)
        .map_err(|_| Trap::new("Read error"))?;
    output[0] = Val::I64(byte as i64);
    Ok(())
}

pub(crate) fn error_set(
    mut caller: Caller<Internal>,
    input: &[Val],
    _output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let offset = input[0].unwrap_i64() as usize;
    let length = match memory!(data).block_length(offset) {
        Some(x) => x,
        None => return Err(Trap::new("Invalid offset in call to error_set")),
    };

    let handle = MemoryBlock { offset, length };
    if handle.offset == 0 {
        plugin!(mut data).clear_error();
        return Ok(());
    }

    let buf = memory!(data).get(handle);
    let s = unsafe { std::str::from_utf8_unchecked(buf) };
    plugin!(mut data).set_error(s);
    Ok(())
}

pub(crate) fn config_get(
    mut caller: Caller<Internal>,
    input: &[Val],
    output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let offset = input[0].unwrap_i64() as usize;
    let length = match memory!(data).block_length(offset) {
        Some(x) => x,
        None => return Err(Trap::new("Invalid offset in call to config_get")),
    };

    let buf = memory!(data).get((offset, length));
    let str = unsafe { std::str::from_utf8_unchecked(buf) };
    let val = plugin!(data).manifest.as_ref().config.get(str);
    let mem = match val {
        Some(f) => memory!(mut data).alloc_bytes(f.as_bytes())?,
        None => return Err(Trap::new("Invalid config key")),
    };

    output[0] = Val::I64(mem.offset as i64);
    Ok(())
}

pub(crate) fn var_get(
    mut caller: Caller<Internal>,
    input: &[Val],
    output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let offset = input[0].unwrap_i64() as usize;
    let length = match memory!(data).block_length(offset) {
        Some(x) => x,
        None => return Err(Trap::new("Invalid offset in call to var_get")),
    };

    let buf = memory!(data).get((offset, length));
    let str = unsafe { std::str::from_utf8_unchecked(buf) };
    let val = data.vars.get(str);
    let mem = match val {
        Some(f) => memory!(mut data).alloc_bytes(&f)?,
        None => {
            output[0] = Val::I64(0);
            return Ok(());
        }
    };

    output[0] = Val::I64(mem.offset as i64);
    Ok(())
}

pub(crate) fn var_set(
    mut caller: Caller<Internal>,
    input: &[Val],
    _output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();

    let mut size = 0;
    for v in data.vars.values() {
        size += v.len();
    }

    let offset1 = input[1].unwrap_i64() as usize;

    // If the store is larger than 100MB then stop adding things
    if size > 1024 * 1024 * 100 && offset1 != 0 {
        return Err(Trap::new("Variable store is full"));
    }

    let offset = input[0].unwrap_i64() as usize;
    let length = match memory!(data).block_length(offset) {
        Some(x) => x,
        None => return Err(Trap::new("Invalid offset in call to var_set")),
    };

    let kbuf = memory!(data).get((offset, length));
    let kstr = unsafe { std::str::from_utf8_unchecked(kbuf) };

    let length1 = match memory!(data).block_length(offset) {
        Some(x) => x,
        None => return Err(Trap::new("Invalid offset in call to var_set")),
    };

    if offset1 == 0 {
        data.vars.remove(kstr);
        return Ok(());
    }

    let vbuf = memory!(data).get((offset1, length1));

    data.vars.insert(kstr.to_string(), vbuf.to_vec());
    Ok(())
}

#[derive(serde::Serialize, serde::Deserialize)]
struct HttpRequest {
    url: String,
    #[serde(default)]
    header: std::collections::BTreeMap<String, String>,
    method: Option<String>,
}

pub(crate) fn http_request(
    #[allow(unused_mut)] mut caller: Caller<Internal>,
    input: &[Val],
    output: &mut [Val],
) -> Result<(), Trap> {
    #[cfg(not(feature = "http"))]
    {
        let _ = (caller, input, output);
        panic!("HTTP requests have been disabled");
    }

    #[cfg(feature = "http")]
    {
        use std::io::Read;
        let data: &mut Internal = caller.data_mut();
        let offset = input[0].unwrap_i64() as usize;

        let length = match memory!(data).block_length(offset) {
            Some(x) => x,
            None => return Err(Trap::new("Invalid offset in call to config_get")),
        };
        let buf = memory!(data).get((offset, length));
        let req: HttpRequest =
            serde_json::from_slice(buf).map_err(|_| Trap::new("Invalid http request"))?;

        let mut r = ureq::request(req.method.as_deref().unwrap_or("GET"), &req.url);

        for (k, v) in req.header.iter() {
            r = r.set(k, v);
        }

        let mut r = r
            .call()
            .map_err(|e| Trap::new(format!("{:?}", e)))?
            .into_reader();

        let mut buf = Vec::new();
        r.read_to_end(&mut buf)
            .map_err(|e| Trap::new(format!("{:?}", e)))?;

        let mem = memory!(mut data).alloc_bytes(buf)?;

        output[0] = Val::I64(mem.offset as i64);
        Ok(())
    }
}

pub(crate) fn length(
    mut caller: Caller<Internal>,
    input: &[Val],
    output: &mut [Val],
) -> Result<(), Trap> {
    let data: &mut Internal = caller.data_mut();
    let offset = input[0].unwrap_i64() as usize;
    let length = match memory!(data).block_length(offset) {
        Some(x) => x,
        None => return Err(Trap::new("Unable to find length for offset")),
    };
    output[0] = Val::I64(length as i64);
    Ok(())
}
