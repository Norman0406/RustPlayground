trait Shape {
    fn area(&self) -> f64;
    fn perimeter(&self) -> f64;
    fn area_per_perimeter(&self) -> f64 {
        return self.area() / self.perimeter();
    }
}

struct Circle {
    radius: f64
}

impl Shape for Circle {
    fn area(&self) -> f64 {
        std::f64::consts::PI * self.radius.powi(2)
    }

    fn perimeter(&self) -> f64 {
        2.0 * std::f64::consts::PI * self.radius
    }
}

struct Rectangle {
    width: f64,
    height: f64
}

impl Shape for Rectangle {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn perimeter(&self) -> f64 {
        2.0 * self.width + 2.0 * self.height
    }
}

fn main() {
    // create plain objects
    let c1 = Circle { radius: 1.0 };
    let c2 = Circle { radius: 2.0 };
    let r1 = Rectangle { width: 1.0, height: 1.0 };
    let r2 = Rectangle { width: 2.0, height: 3.0 };

    println!("=== Plain ===");
    println!("c1: area {}, perimeter {}, area_per_perimeter {}", c1.area(), c1.perimeter(), c1.area_per_perimeter());
    println!("c2: area {}, perimeter {}, area_per_perimeter {}", c2.area(), c2.perimeter(), c2.area_per_perimeter());
    println!("r1: area {}, perimeter {}, area_per_perimeter {}", r1.area(), r1.perimeter(), r1.area_per_perimeter());
    println!("r2: area {}, perimeter {}, area_per_perimeter {}", r2.area(), r2.perimeter(), r2.area_per_perimeter());

    // borrow and store them in a vector
    let mut vec_of_refs: Vec<&dyn Shape> = Vec::new();
    vec_of_refs.push(&c1);
    vec_of_refs.push(&c2);
    vec_of_refs.push(&r1);
    vec_of_refs.push(&r2);

    println!("=== Refs ===");
    for shape in vec_of_refs {
        println!("Shape: area {}, perimeter {}, area_per_perimeter {}", shape.area(), shape.perimeter(), shape.area_per_perimeter());
    }

    // create a vector of boxed objects (i.e. pointers)
    let mut vec_of_boxes: Vec<Box<dyn Shape>> = Vec::new();

    // create boxed objects by allocating new memory and moving the values
    vec_of_boxes.push(Box::new(c1));
    vec_of_boxes.push(Box::new(c2));
    vec_of_boxes.push(Box::new(r1));
    vec_of_boxes.push(Box::new(r2));

    println!("=== Boxes ===");
    for shape in vec_of_boxes {
        println!("Shape: area {}, perimeter {}, area_per_perimeter {}", shape.area(), shape.perimeter(), shape.area_per_perimeter());
    }
}
