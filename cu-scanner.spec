Name: cu-scanner
Version: 1.0
Release: 1

Summary: A tool for convert csaf file to oval format xml file.

License: Mulan PSL v2
Source: %{name}.tar.gz

BuildRequires: cargo rust openssl-devel

Requires: libgcc openssl-libs glibc

%description
cu-scanner is a tool specifically designed for the field of cybersecurity. Its core function is to collect, organize, and uniquely process relevant information based on files or data interfaces related to CVE (Common Vulnerabilities and Exposures) and security advisories, thereby rapidly generating XML files in OVAL (Open Vulnerability and Assessment Language) format.

%prep
%setup -n %{name}-%{version} -c

%build
cargo build --release

%install
rm -rf %{buildroot}
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/etc/cu-scanner
mkdir -p %{buildroot}/usr/lib/systemd/system

install -m 0755 target/release/%{name} %{buildroot}/usr/bin/
install -m 0644 config/%{name}.toml  %{buildroot}/etc/cu-scanner/
install -m 0644 %{name}.service %{buildroot}/usr/lib/systemd/system/
install -m 0644 config/epoch_data.json %{buildroot}/etc/cu-scanner/

%files
/usr/bin/%{name}
/etc/cu-scanner/%{name}.toml
/etc/cu-scanner/epoch_data.json
/usr/lib/systemd/system/%{name}.service

%changelog
* Mon Nov 10 2025 Cao Jingbo <caojb22@chinaunicom.cn> - 1.0.0-1
- Initial package.
