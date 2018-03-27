
//use std::ops::Add;
use utils::PointF;

#[derive(Debug)]
pub struct SMatrix{
    _11 : f64, _12: f64, _13: f64,
    _21 : f64, _22: f64, _23: f64,
    _31 : f64, _32: f64, _33: f64,
}

impl SMatrix {
    pub fn new() -> SMatrix {
        SMatrix{
            _11:0.0, _12:0.0, _13:0.0,
            _21:0.0, _22:0.0, _23:0.0,
            _31:0.0, _32:0.0, _33:0.0
        }
    }
}

#[derive(Debug)]
pub struct Matrix{
    matrix: SMatrix
}

impl Matrix {
    pub fn new() -> Matrix {
        let mut m = Matrix{ matrix: SMatrix::new() };
        m.identity();
        m
    }

    /** 将矩阵初始化为单位矩阵 */
    pub fn identity(&mut self) {
        self.matrix._11=1.0; self.matrix._12=0.0; self.matrix._13=0.0;
        self.matrix._21=0.0; self.matrix._22=1.0; self.matrix._23=0.0;
        self.matrix._31=0.0; self.matrix._32=0.0; self.matrix._33=1.0;
    }

    //创建一个变换矩阵
    pub fn translate(&mut self, x: f64, y: f64){
        let mut mat = SMatrix::new();
        mat._11 = 1.0; mat._12 = 0.0; mat._13 = 0.0;
	    mat._21 = 0.0; mat._22 = 1.0; mat._23 = 0.0;
	    mat._31 = x;    mat._32 = y;    mat._33 = 1.0;
	    //and multiply
        self.matrix_multiply(mat);
    }

    pub fn matrix_multiply(&mut self, m_in:SMatrix) {
        let mut mat_temp = SMatrix::new();
        
        //first row
        mat_temp._11 = (self.matrix._11*m_in._11) + (self.matrix._12*m_in._21) + (self.matrix._13*m_in._31);
        mat_temp._12 = (self.matrix._11*m_in._12) + (self.matrix._12*m_in._22) + (self.matrix._13*m_in._32);
        mat_temp._13 = (self.matrix._11*m_in._13) + (self.matrix._12*m_in._23) + (self.matrix._13*m_in._33);

        //second
        mat_temp._21 = (self.matrix._21*m_in._11) + (self.matrix._22*m_in._21) + (self.matrix._23*m_in._31);
        mat_temp._22 = (self.matrix._21*m_in._12) + (self.matrix._22*m_in._22) + (self.matrix._23*m_in._32);
        mat_temp._23 = (self.matrix._21*m_in._13) + (self.matrix._22*m_in._23) + (self.matrix._23*m_in._33);

        //third
        mat_temp._31 = (self.matrix._31*m_in._11) + (self.matrix._32*m_in._21) + (self.matrix._33*m_in._31);
        mat_temp._32 = (self.matrix._31*m_in._12) + (self.matrix._32*m_in._22) + (self.matrix._33*m_in._32);
        mat_temp._33 = (self.matrix._31*m_in._13) + (self.matrix._32*m_in._23) + (self.matrix._33*m_in._33);

        self.matrix = mat_temp;
    }

    pub fn scale(&mut self, x_scale: f64, y_scale: f64){
        let mut mat = SMatrix::new();
        mat._11 = x_scale; mat._12 = 0.0; mat._13 = 0.0;
	    mat._21 = 0.0; mat._22 = y_scale; mat._23 = 0.0;
	    mat._31 = 0.0; mat._32 = 0.0; mat._33 = 1.0;
	    //and multiply
        self.matrix_multiply(mat);
    }

    pub fn rotate(&mut self, rot: f64){
        let mut mat = SMatrix::new();
        let sin = f64::sin(rot);
        let cos = f64::cos(rot);
        mat._11 = cos; mat._12 = sin; mat._13 = 0.0;
	    mat._21 = -sin; mat._22 = cos; mat._23 = 0.0;
	    mat._31 = 0.0; mat._32 = 0.0; mat._33 = 1.0;
	    //and multiply
        self.matrix_multiply(mat);
    }

    pub fn transform_points(&self, points: &mut Vec<PointF>){
        for point in points {
            let temp_x =(self.matrix._11*point.x) + (self.matrix._21*point.y) + (self.matrix._31);
		    let temp_y = (self.matrix._12*point.x) + (self.matrix._22*point.y) + (self.matrix._32);
		    point.x = temp_x;
            point.y = temp_y;
        }
    }
}

// impl Add for Matrix {
    
// }