use crate::core::matrix::*;
use crate::misc::utils::*;
use crate::core::vector::Vec4;
use crate::objects::object::*;
use crate::ray_tracing::intersection::Intersection;
use crate::materials::material::*;
use crate::objects::group::Group;
use crate::ray_tracing::ray::Ray;
use std::any::Any;

#[derive(Debug, PartialEq, Clone)]
pub struct SmoothTriangle {
    pub p1: Vec4,
    pub p2: Vec4,
    pub p3: Vec4,
    pub n1: Vec4,
    pub n2: Vec4,
    pub n3: Vec4,
    pub e1: Vec4,
    pub e2: Vec4,
    pub material: Material,
    pub parent_inverses: Vec<Matrix4x4>,
    pub parent_material: Option<Material>,
}

impl SmoothTriangle {
    //Instantiates a default smooth triangle
    pub fn default() -> SmoothTriangle {
        SmoothTriangle {
            p1: Vec4(0.0, 1.0, 0.0, 1.0),
            p2: Vec4(-1.0, 0.0, 0.0, 1.0),
            p3: Vec4(1.0, 0.0, 0.0, 1.0),
            n1: Vec4(0.0, 1.0, 0.0, 0.0),
            n2: Vec4(-1.0, 0.0, 0.0, 0.0),
            n3: Vec4(1.0, 0.0, 0.0, 0.0),
            e1: Vec4(-1.0, -1.0, 0.0, 0.0),
            e2: Vec4(1.0, -1.0, 0.0, 0.0),
            material: Material::default(),
            parent_inverses: vec![],
            parent_material: None,
        }
    }

    pub fn new(p1: Vec4, p2: Vec4, p3: Vec4, n1: Vec4, n2: Vec4, n3: Vec4, material: Material) -> SmoothTriangle {
        SmoothTriangle {
           e1: &p2 - &p1,
           e2: &p3 - &p1,
           p1,
           p2,
           p3,
           n1,
           n2,
           n3,
           material,
           parent_inverses: vec![],
           parent_material: None,
        }
    }
}

impl Object for SmoothTriangle {
    //Returns the smooth triangle material
    fn get_material(&self) -> &Material {
        &self.material
    }

    //Returns the smooth triangle inverse
    fn get_inverse(&self) -> &Matrix4x4 {
        &IDENTITY
    }

    //Intersects a ray with a smooth triangle
    fn intersect(&self, ray: &Ray) -> Option<Vec<Intersection>> {
        let dir_cross_e2 = &ray.direction * &self.e2;
        let det = Vec4::dot(&self.e1, &dir_cross_e2);
        if det.abs() <= EPSILON_BUMP {
            return None;
        }
        let f = 1.0 / det;
        let p1_to_origin = &ray.origin - &self.p1;
        let u = f * (Vec4::dot(&p1_to_origin, &dir_cross_e2));
        if u < 0.0 || u > 1.0 {
            return None;
        }
        let origin_cross_e1 = p1_to_origin * &self.e1;
        let v = f * Vec4::dot(&ray.direction, &origin_cross_e1);
        if v < 0.0 || (u + v) > 1.0 {
            return None;
        }
        let t = f * Vec4::dot(&self.e2, &origin_cross_e1);
        Some(
            vec![
                Intersection::new_uv(
                    t,
                    Ray::position(&ray, t),
                    self.normal(&Ray::position(&ray, t), Some(u), Some(v)),
                    self,
                    u,
                    v,
                )
            ]
        )
    }

    //Finds the normal of a given point on a smooth triangle
    fn normal(&self, _world_point: &Vec4, u: Option<f32>, v: Option<f32>) -> Vec4 {
        normal_to_world(&self.parent_inverses, &(&self.n2 * u.unwrap() + &self.n3 * v.unwrap() + &self.n1 * (1.0 - u.unwrap() - v.unwrap())).normalize())
    }

    fn get_parent_inverses(&self) -> &Vec<Matrix4x4> {
        &self.parent_inverses
    }

    fn push_parent_inverse(&mut self, inverse: Matrix4x4) {
        self.parent_inverses.push(inverse);
    }

    fn get_parent_material(&self) -> &Option<Material> {
        &self.parent_material
    }

    fn set_parent_material(&mut self, material: &Material) {
        self.parent_material = Some(material.clone());
    }

    fn add_to_group(mut self, group: &mut Group) {
        self.push_parent_inverse(group.get_inverse().clone());
        self.set_parent_material(&group.material);
        group.objects.push(Box::new(self));
    }

    fn eq(&self, other: &dyn Object) -> bool {
        other.as_any().downcast_ref::<Self>().map_or(false, |x| x == self)
    }

    fn as_any(&self) -> &dyn Any { self }
}
