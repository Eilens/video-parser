use md5::{Md5, Digest};
use std::time::{SystemTime, UNIX_EPOCH};

fn md5_hex(data: &[u8]) -> String {
    let mut hasher = Md5::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

fn md5_bytes(data: &[u8]) -> Vec<u8> {
    let mut hasher = Md5::new();
    hasher.update(data);
    hasher.finalize().to_vec()
}

// Port of M(aa)
fn m_transform(aa: &str) -> Vec<u8> {
    let r = [
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, Some(0), Some(1), Some(2), Some(3), Some(4), Some(5), Some(6), Some(7), Some(8), Some(9), None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None, None,
        None, None, Some(10), Some(11), Some(12), Some(13), Some(14), Some(15)
    ];
    
    let l = aa.len() >> 1;
    let mut u = vec![0u8; l];
    let mut c = 0;
    let mut b = 0;
    
    let aa_bytes = aa.as_bytes();
    
    while b < (l << 1) {
        let code1 = aa_bytes[b] as usize;
        let v1 = if code1 < r.len() { r[code1] } else { None };
        b += 1;
        
        let code2 = aa_bytes[b] as usize;
        let v2 = if code2 < r.len() { r[code2] } else { None };
        b += 1;
        
        if let (Some(a), Some(val_b)) = (v1, v2) {
             u[c] = ((a << 4) | val_b) as u8;
             c += 1;
        }
    }
    
    u
}

// Port of u(aa, bb)
fn u_transform(aa: &[u8], bb: Option<&str>) -> String {
    let bb_str = bb.unwrap_or("Dkdpgh4ZKsQB80/Mfvw36XI1R25-WUAlEi7NLboqYTOPuzmFjJnryx9HVGcaStCe=");
    let bb_bytes = bb_str.as_bytes();
    let w = aa.len();
    let mut v = String::new();
    
    for i in (0..w).step_by(3) {
        if i + 2 >= w { break; } 
        let n1 = aa[i] as u32;
        let n2 = aa[i+1] as u32;
        let n3 = aa[i+2] as u32;
        
        let m = (n1 << 16) | (n2 << 8) | n3;
        
        v.push(bb_bytes[((m >> 18) & 63) as usize] as char);
        v.push(bb_bytes[((m >> 12) & 63) as usize] as char);
        v.push(bb_bytes[((m >> 6) & 63) as usize] as char);
        v.push(bb_bytes[(m & 63) as usize] as char);
    }
    v
}

// Port of K(aa, z)
// aa: mixing key (string/bytes), z: input bytes
fn k_transform(aa: &[u8], z: &[u8]) -> Vec<u8> {
    let mut q: Vec<u8> = (0..=255).collect();
    let mut c = 0;
    
    // Key setup
    for y in 0..256 {
        let p = if !aa.is_empty() { aa[y % aa.len()] as usize } else { 0 };
        c = (c + q[y] as usize + p) % 256;
        q.swap(y, c);
    }
    
    let mut d = 0;
    let mut c = 0;
    let mut p_out = Vec::new();
    
    for &byte in z {
        d = (d + 1) % 256;
        c = (c + q[d] as usize) % 256;
        q.swap(d, c);
        let k = q[(q[d] as usize + q[c] as usize) % 256];
        p_out.push(byte ^ k);
    }
    
    p_out
}

// Port of s(aa, bb)
fn s_transform(aa: u32, bb: u32) -> Vec<u8> {
    let mut q = vec![0u8; 3];
    q[0] = ((aa >> 8) & 255) as u8;
    q[1] = (aa & 255) as u8;
    q[2] = (bb & 255) as u8;
    q
}

// Port of H(...)
fn h_transform(params: Vec<u8>) -> Vec<u8> {
    // Expected params size: 19
    let mut k = vec![0u8; 19];
    // Mapping based on python H args:
    // H(QQ, WW, EE, RR, TT, YY, UU, II, OO, PP, AA, SS, DD, FF, GG, HH, JJ, KK, LL)
    // Indexes:
    // 0:QQ, 1:WW, 2:EE, 3:RR, 4:TT, 5:YY, 6:UU, 7:II, 8:OO, 9:PP, 10:AA, 11:SS, 12:DD, 13:FF, 14:GG, 15:HH, 16:JJ, 17:KK, 18:LL
    // Python code:
    // K[0]=QQ, K[1]=AA (idx 10), K[2]=WW (idx 1), K[3]=SS (idx 11), K[4]=EE (idx 2), K[5]=DD (idx 12)
    // K[6]=RR (idx 3), K[7]=FF (idx 13), K[8]=TT (idx 4), K[9]=GG (idx 14), K[10]=YY (idx 5)
    // K[11]=HH (idx 15), K[12]=UU (idx 6), K[13]=JJ (idx 16), K[14]=II (idx 7), K[15]=KK (idx 17)
    // K[16]=OO (idx 8), K[17]=LL (idx 18), K[18]=PP (idx 9)
    
    if params.len() < 19 { return k; }
    
    k[0] = params[0];
    k[1] = params[10];
    k[2] = params[1];
    k[3] = params[11];
    k[4] = params[2];
    k[5] = params[12];
    k[6] = params[3];
    k[7] = params[13];
    k[8] = params[4];
    k[9] = params[14];
    k[10] = params[5];
    k[11] = params[15];
    k[12] = params[6];
    k[13] = params[16];
    k[14] = params[7];
    k[15] = params[17];
    k[16] = params[8];
    k[17] = params[18];
    k[18] = params[9];
    
    k
}

// Port of C(r)
fn c_transform(r: &[u8]) -> u8 {
    let mut a = r[0] ^ r[1];
    for i in 2..r.len() {
        a = a ^ r[i];
    }
    a
}

// Port of f(...)
fn f_transform(aa: u8, l1: &[u8], l2: &[u8], l3: &[u8]) -> Vec<u8> {
    let d = 1 >> 8 & 255; // 1/256 in int is 0
    let e = 1 & 255;
    
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
    let t_ = now as u32;
    
    let f_byte = ((t_ >> 24) & 255) as u8;
    let g_byte = ((t_ >> 16) & 255) as u8;
    let h_byte = ((t_ >> 8) & 255) as u8;
    let i_byte = (t_ & 255) as u8;
    
    // Seeded random simulation? Python uses random.randint around fixed seed? 
    // "m = random.randint(536919696 - 10000, 536919696 + 10000)"
    // Let's pick a fixed middle value for deterministic behavior or use rand if we import it.
    // Since we didn't add rand crate (or did we? we did in Cargo.toml), let's use it.
    // For now, let's use the static value 536919696 to match typical behavior if rand is not critical.
    // Actually, x-bogus usually requires changing timestamp but fixed seed might be detected?
    // Using fixed value 536919696.
    let m = 536919696u32; 
    
    let f1_byte = ((m >> 24) & 255) as u8;
    let g1_byte = ((m >> 16) & 255) as u8;
    let h1_byte = ((m >> 8) & 255) as u8;
    let i1_byte = (m & 255) as u8;
    
    let r = vec![
        64, d as u8, e as u8, aa,
        l1[14], l1[15],
        l2[14], l2[15],
        l3[14], l3[15],
        f_byte, g_byte, h_byte, i_byte,
        f1_byte, g1_byte, h1_byte, i1_byte
    ];
    
    let v = c_transform(&r);
    
    let mut r_final = Vec::new();
    // Logic: [r[i] for i in range(len(r)) if i % 2 == 0] + [v] + [r[i] for i in range(len(r)) if i % 2 == 1]
    
    for index in 0..r.len() {
        if index % 2 == 0 { r_final.push(r[index]); }
    }
    r_final.push(v);
    for index in 0..r.len() {
        if index % 2 == 1 { r_final.push(r[index]); }
    }
    
    r_final
}

pub fn get_x_b(url: &str, ua: &str) -> String {
    let data = "d41d8cd98f00b204e9800998ecf8427e";
    
    // Y = M(md5(bytes(M(md5(url.encode()).hexdigest()))).hexdigest())
    let url_md5 = md5_hex(url.as_bytes());
    let url_m_bytes = m_transform(&url_md5);
    let url_m_md5 = md5_hex(&url_m_bytes);
    let y_vec = m_transform(&url_m_md5);
    
    // L = M(md5(bytes(M(md5(data.encode()).hexdigest()))).hexdigest())
    let data_md5 = md5_hex(data.as_bytes());
    let data_m_bytes = m_transform(&data_md5);
    let data_m_md5 = md5_hex(&data_m_bytes);
    let l_vec = m_transform(&data_m_md5);
    
    // A = M(md5(u(K(s(1, 12), ua), ...).encode()).hexdigest())
    let s_res = s_transform(1, 12);
    let k_res = k_transform(&s_res, ua.as_bytes());
    let u_res = u_transform(&k_res, Some("ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/="));
    let a_md5 = md5_hex(u_res.as_bytes());
    let a_vec = m_transform(&a_md5);
    
    let e = f_transform(12, &y_vec, &l_vec, &a_vec);
    
    let a_h = h_transform(e);
    
    let b_k = k_transform(&[255], &a_h);
    
    let mut c_bytes = vec![2, 255];
    c_bytes.extend(b_k);
    
    // Need to base64 encode c which is binary? 
    // In python: c = chr(2) + chr(255) + b. 
    // b comes from K(...) which returns STRING in python code `return P` but my K returns Vec<u8>.
    // The python K function essentially does `P += chr(...)`. So it returns a string of bytes.
    // So c is a string. `u(c)` treats inputs as chars.
    // In Rust, we work with bytes. 
    
    let d = u_transform(&c_bytes, None);
    d
}
