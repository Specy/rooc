pub fn divide_matrix_row_by(matrix: &mut [Vec<f64>], row: usize, value: f64) {
    for i in 0..matrix[row].len() {
        matrix[row][i] /= value;
    }
}
