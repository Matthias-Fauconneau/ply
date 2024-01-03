fn main() {
	use ::ply::ply::*;
	let ref path = std::env::args().skip(1).next().unwrap_or("icosahedron.ply".to_owned());
	let (vertices, faces) = match path.as_ref() {
		"icosahedron.ply" => {
			fn sqrt(x: f32) -> f32 { f32::sqrt(x) }
			let v5 = sqrt(5.);
			let mut vertices = Vec::new();
			for v in [[1.,0.,0.],[1./v5,2./v5,0.],[1./v5,(1.-1./v5)/2.,sqrt((1.+1./v5)/2.)],[1./v5,(-1.-1./v5)/2.,sqrt((1.-1./v5)/2.)]].into_iter().map(|[x,y,z]|[[x,y,z], [-x,-y,-z], [x,y,-z], [-x,-y,z]]).flatten() {
				if !vertices.contains(&v) { vertices.push(v); }
			}
			let faces = {
				let vertices = &vertices;
				let mut v = (0..vertices.len()).map(move |a| (0..vertices.len()).filter(move |&b| b!=a).map(move |b| (0..vertices.len()).filter(move |&c| c!=b && c!=a).map(move |c| [a,b,c]))).flatten().flatten().map(|[a,b,c]| {
					use vector::{cross, dot, sq, vec3};
					#[allow(non_snake_case)] let [A,B,C] = [a,b,c].map(|i| vec3::from(vertices[i]));
					let n = cross(B-A,C-A);
					let [a,b,c] = if dot(n, A+B+C) > 0. { [a,b,c] } else { [c,b,a] };
					(f32::abs(sq(n)), [a,b,c])
				}).collect::<Vec<_>>();
				v.sort_by(|(a, _),(b,_)| a.total_cmp(b));
				v.into_iter().map(|(_,[a,b,c])| [a as u32, b as _, c as _]).take(20).collect::<Vec<_>>()
			};
			//let faces = [[1,11,7], [1,7,6], [1,6,10], [1,10,3], [1,3,11], [4,8,0], [5,4,0], [9,5,0], [2,9,0], [8,2,0], [11,9,7], [7,2,6], [6,8,10], [10,4,3], [3,5,11], [4,10,8], [5,3,4], [9,11,5], [2,7,9], [8,6,2]];
			//assert_eq!(*faces.iter().flatten().max().unwrap(), vertices.len() as u32-1);
			(vertices, faces) //Vec::from(faces)
		},
		path => {
			let ref mut file = std::io::BufReader::new(std::fs::File::open(path).unwrap());
			use ::ply::parser::*;
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
			let vertices = Parser::<Vertex>::new().read_payload_for_element(file, &vertex.1, header).unwrap().into_iter().map(|Vertex(v)| v).collect();
			let faces = Parser::<Face>::new().read_payload_for_element(file, &face.1, header).unwrap().into_iter().map(|Face(face)| face).collect();
			(vertices, faces)
		}
	};

	use ::ply::writer::Writer;
	Writer::new().write_ply(&mut std::fs::File::/*create_new*/options().write(true).create_new(true).open(path.to_owned()+"b").unwrap(), &mut Ply::<DefaultElement>{
			header: Header{
				encoding: Encoding::BinaryLittleEndian,
				elements: [
					("vertex".to_string(), ElementDef{name: "vertex".to_string(), count: 0, properties: ["x","y","z"].iter().map(|k| (k.to_string(), PropertyDef::new(k.to_string(), PropertyType::Scalar(ScalarType::Float)))).into_iter().collect()}),
					("face".to_string(), ElementDef{name: "face".to_string(), count: 0, properties: [("vertex_indices".to_string(), PropertyDef::new("vertex_indices".to_string(), PropertyType::List(ScalarType::UChar, ScalarType::UInt)))].into_iter().collect()}),
				].into_iter().collect(),
				..Header::new()
			},
			payload: [
				("vertex".to_string(), vertices.into_iter().map(|v| ["x","y","z"].iter().zip(v).map(|(k,v)| (k.to_string(), Property::Float(v))).collect()).collect()),
				("face".to_string(), faces.into_iter().map(|f| [("vertex_indices".to_string(), Property::ListUInt(f.into()))].into_iter().collect()).collect()),
			].into_iter().collect()
	}).unwrap();
}
