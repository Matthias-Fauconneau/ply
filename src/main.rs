fn main() {
	let ref path = std::env::args().skip(1).next().unwrap();
	let ref mut file = std::io::BufReader::new(std::fs::File::open(path).unwrap());
	use ::ply::{parser::*, ply::*};
	let ref header = Parser::<DefaultElement>::new().read_header(file).unwrap();
	assert_eq!(header.encoding, Encoding::Ascii);
	let [vertex, face] = header.elements.iter().collect::<Vec<_>>().try_into().unwrap();
	assert_eq!(vertex.0,"vertex");
	assert_eq!(vertex.1.name,"vertex");
	assert_eq!(vertex.1.properties,["x","y","z"].map(|name| (name.into(), PropertyDef{name: name.into(), data_type: PropertyType::Scalar(ScalarType::Float)})).into_iter().collect());
	assert_eq!(face.0,"face");
	assert_eq!(face.1.name,"face");
	//assert_eq!(face.1.properties, [("vertex_indices".into(), PropertyDef{name: "vertex_indices".into(), data_type: PropertyType::List(ScalarType::UInt, ScalarType::UInt)})].into_iter().collect());
	let [vertex_indices] = face.1.properties.iter().collect::<Vec<_>>().try_into().unwrap();
	assert_eq!(vertex_indices.0, "vertex_indices");
	assert_eq!(vertex_indices.1.name, "vertex_indices");
	assert!(matches!(vertex_indices.1.data_type, PropertyType::List(ScalarType::UInt|ScalarType::UChar, ScalarType::UInt|ScalarType::Int)), "{vertex_indices:?}");

	struct Vertex([f32; 3]);
	impl PropertyAccess for Vertex {
		fn new() -> Self { unsafe{#[allow(invalid_value)] std::mem::MaybeUninit::uninit().assume_init()} } // All fields overwritten by set_property
		fn set_property(&mut self, key: String, value: Property) { match (key.as_ref(), value) {
			("x", Property::Float(x)) => self.0[0] = x,
			("y", Property::Float(y)) => self.0[1] = y,
			("z", Property::Float(z)) => self.0[2] = z,
			_ =>  unreachable!(),
		}}
	}
	struct Face([u32; 3]);
	impl PropertyAccess for Face {
		fn new() -> Self { unsafe{#[allow(invalid_value)] std::mem::MaybeUninit::uninit().assume_init()} } // All fields overwritten by set_property
		fn set_property(&mut self, _: String, value: Property) { match value {
			Property::ListUInt(vec) => self.0 = vec.try_into().unwrap(),
			Property::ListInt(vec) => self.0 = <[_; 3]>::try_from(vec).unwrap().map(|int| int as _),
			_ => unreachable!("{value:?}")
		}}
	}
	let vertices = Parser::<Vertex>::new().read_payload_for_element(file, &vertex.1, header).unwrap();
	let faces = Parser::<Face>::new().read_payload_for_element(file, &face.1, header).unwrap();

	use ::ply::writer::Writer;
	Writer::new().write_ply(&mut std::fs::File::/*create_new*/options().write(true).create_new(true).open(path.to_owned()+"b").unwrap(), &mut Ply::<DefaultElement>{
			header: Header{
				encoding: Encoding::BinaryLittleEndian,
				elements: [
					("vertex".to_string(), ElementDef{name: "vertex".to_string(), count: 0, properties: ["x","y","z"].iter().map(|k| (k.to_string(), PropertyDef::new(k.to_string(), PropertyType::Scalar(ScalarType::Float)))).into_iter().collect()}),
					("face".to_string(), ElementDef{name: "face".to_string(), count: 0, properties: [("vertex_indices".to_string(), PropertyDef::new("vertex_indices".to_string(), PropertyType::List(ScalarType::UInt, ScalarType::UInt)))].into_iter().collect()}),
				].into_iter().collect(),
				..Header::new()
			},
			payload: [
				("vertex".to_string(), vertices.iter().map(|&Vertex(v)| ["x","y","z"].iter().zip(v).map(|(k,v)| (k.to_string(), Property::Float(v))).collect()).collect()),
				("face".to_string(), faces.iter().map(|Face(face)| [("vertex_indices".to_string(), Property::ListUInt(face.into()))].into_iter().collect()).collect()),
			].into_iter().collect()
	}).unwrap();
}