////////////////////////////////////////////////////////////////////////////////
// 版本号排序
////////////////////////////////////////////////////////////////////////////////

/// 版本号比对
pub(crate) fn cmp_version(a: &str, b: &str, compare_len: bool) -> std::cmp::Ordering {
  let mut a_version_str = a;
  if a.contains(' ') {
      let temp: Vec<&str> = a.split(' ').collect();
      a_version_str = temp.first().unwrap_or(&"");
  }
  let mut b_version_str = b;
  if b.contains(' ') {
      let temp: Vec<&str> = b.split(' ').collect();
      b_version_str = temp.first().unwrap_or(&"");
  }
  let arr1: Vec<&str> = a_version_str.split('.').collect();
  let arr2: Vec<&str> = b_version_str.split('.').collect();
  let length = std::cmp::min(arr1.len(), arr2.len());
  for i in 0..length {
      let num1: usize = arr1.get(i).unwrap_or(&"0").parse().unwrap_or(0);
      let num2: usize = arr2.get(i).unwrap_or(&"0").parse().unwrap_or(0);
      let re = num1.cmp(&num2);
      if !re.is_eq() {
          return re;
      }
  }
  if compare_len {
      arr1.len().cmp(&arr2.len())
  } else {
      std::cmp::Ordering::Equal
  }
}