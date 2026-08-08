use quick_xml::se::Serializer;
use serde::Serialize;

/// 模拟 OVAL 根元素：默认命名空间 + 5 个前缀命名空间声明
#[derive(Debug, Serialize)]
#[serde(rename = "oval_definitions")]
struct OvalDefinitions {
    #[serde(rename = "@xmlns")]
    xmlns: String,
    #[serde(rename = "@xmlns:oval")]
    xmlns_oval: String,
    #[serde(rename = "@xmlns:unix-def")]
    xmlns_unix_def: String,
    #[serde(rename = "@xmlns:red-def")]
    xmlns_red_def: String,
    #[serde(rename = "@xmlns:ind-def")]
    xmlns_ind_def: String,
    #[serde(rename = "@xmlns:xsi")]
    xmlns_xsi: String,
    #[serde(rename = "@xsi:schemaLocation")]
    schema_location: String,

    generator: Generator,
    tests: Tests,
}

#[derive(Debug, Serialize)]
struct Generator {
    #[serde(rename = "oval:product_name")]
    product_name: String,
    #[serde(rename = "oval:schema_version")]
    schema_version: String,
}

/// <tests> 内是异构元素列表：red-def:rpminfo_test / red-def:rpmverifyfile_test
/// 用外部标签枚举（externally tagged enum）实现不同标签名
#[derive(Debug, Serialize)]
struct Tests {
    #[serde(rename = "$value")]
    items: Vec<Test>,
}

#[derive(Debug, Serialize)]
enum Test {
    #[serde(rename = "red-def:rpminfo_test")]
    RpmInfo {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "@check")]
        check: String,
        #[serde(rename = "red-def:object")]
        object: Ref,
        #[serde(rename = "red-def:state")]
        state: Ref,
    },
    #[serde(rename = "red-def:rpmverifyfile_test")]
    RpmVerifyFile {
        #[serde(rename = "@id")]
        id: String,
        #[serde(rename = "red-def:object")]
        object: Ref,
    },
}

#[derive(Debug, Serialize)]
struct Ref {
    #[serde(rename = "@object_ref", skip_serializing_if = "Option::is_none")]
    object_ref: Option<String>,
    #[serde(rename = "@state_ref", skip_serializing_if = "Option::is_none")]
    state_ref: Option<String>,
}

fn build() -> OvalDefinitions {
    OvalDefinitions {
        xmlns: "http://oval.mitre.org/XMLSchema/oval-definitions-5".into(),
        xmlns_oval: "http://oval.mitre.org/XMLSchema/oval-common-5".into(),
        xmlns_unix_def: "http://oval.mitre.org/XMLSchema/oval-definitions-5#unix".into(),
        xmlns_red_def: "http://oval.mitre.org/XMLSchema/oval-definitions-5#linux".into(),
        xmlns_ind_def: "http://oval.mitre.org/XMLSchema/oval-definitions-5#independent".into(),
        xmlns_xsi: "http://www.w3.org/2001/XMLSchema-instance".into(),
        schema_location: "http://oval.mitre.org/XMLSchema/oval-definitions-5 oval-definitions-schema.xsd".into(),
        generator: Generator {
            product_name: "cu-scanner".into(),
            schema_version: "5.10".into(),
        },
        tests: Tests {
            items: vec![
                Test::RpmVerifyFile {
                    id: "oval:com.chinaunicom.cuos:tst:202516650001".into(),
                    object: Ref {
                        object_ref: Some("oval:com.chinaunicom.cuos:obj:202516650001".into()),
                        state_ref: None,
                    },
                },
                Test::RpmInfo {
                    id: "oval:com.chinaunicom.cuos:tst:202516650201".into(),
                    check: "at least one".into(),
                    object: Ref {
                        object_ref: Some("oval:com.chinaunicom.cuos:obj:202516650201".into()),
                        state_ref: None,
                    },
                    state: Ref {
                        object_ref: None,
                        state_ref: Some("oval:com.chinaunicom.cuos:ste:202516650201".into()),
                    },
                },
            ],
        },
    }
}

fn main() {
    let doc = build();

    // 方式一：紧凑输出
    let compact = quick_xml::se::to_string(&doc).expect("compact serialize");
    println!("=== 紧凑输出（se::to_string）===\n{}\n", compact);

    // 方式二：美化输出（Serializer 写入 String，调用 .indent()）
    let mut buf = String::new();
    let mut ser = Serializer::new(&mut buf);
    ser.indent(' ', 2);
    doc.serialize(ser).expect("pretty serialize");
    println!("=== 美化输出（Serializer::indent, 2 空格）===\n{}", buf);
}
