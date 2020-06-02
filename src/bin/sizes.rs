#![allow(non_camel_case_types, dead_code)]

#[allow(unused_imports)]
use gleam::gl::{
    GLbitfield, GLclampf, GLenum, GLfloat, GLint, GLsizei, GLsizeiptr, GLuint, GLvoid,
};

#[derive(Copy, Clone, Debug)]
pub struct BufToGl(pub usize);

/// An identifier for a memory buffer allocated by and returned from from GL.
#[derive(Copy, Clone, Debug)]
pub struct BufFromGl(pub usize);

fn main() {
    struct active_texture {
        texture: GLenum,
    }
    println!("{} active_texture", std::mem::size_of::<active_texture>());

    struct bind_buffer {
        target: GLenum,
        buffer: GLuint,
    }
    println!("{} bind_buffer", std::mem::size_of::<bind_buffer>());

    struct bind_texture {
        target: GLenum,
        texture: GLuint,
    }
    println!("{} bind_texture", std::mem::size_of::<bind_texture>());

    struct bind_vertex_array {
        vao: GLuint,
    }
    println!(
        "{} bind_vertex_array",
        std::mem::size_of::<bind_vertex_array>()
    );

    struct buffer_data_untyped {
        target: GLenum,
        size_data: BufToGl,
        usage: GLenum,
    }
    println!(
        "{} buffer_data_untyped",
        std::mem::size_of::<buffer_data_untyped>()
    );

    struct clear_color {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    }
    println!("{} clear_color", std::mem::size_of::<clear_color>());

    struct disable {
        cap: GLenum,
    }
    println!("{} disable", std::mem::size_of::<disable>());

    struct disable_vertex_attrib_array {
        index: GLuint,
    }
    println!(
        "{} disable_vertex_attrib_array",
        std::mem::size_of::<disable_vertex_attrib_array>()
    );

    struct enable {
        cap: GLenum,
    }
    println!("{} enable", std::mem::size_of::<enable>());

    struct enable_vertex_attrib_array {
        index: GLuint,
    }
    println!(
        "{} enable_vertex_attrib_array",
        std::mem::size_of::<enable_vertex_attrib_array>()
    );

    struct gen_buffers {
        n: GLsizei,
        returned: BufFromGl,
    }
    println!("{} gen_buffers", std::mem::size_of::<gen_buffers>());

    struct gen_framebuffers {
        n: GLsizei,
        returned: BufFromGl,
    }
    println!(
        "{} gen_framebuffers",
        std::mem::size_of::<gen_framebuffers>()
    );

    struct gen_queries {
        n: GLsizei,
        returned: BufFromGl,
    }
    println!("{} gen_queries", std::mem::size_of::<gen_queries>());

    struct gen_renderbuffers {
        n: GLsizei,
        returned: BufFromGl,
    }
    println!(
        "{} gen_renderbuffers",
        std::mem::size_of::<gen_renderbuffers>()
    );

    struct gen_textures {
        n: GLsizei,
        returned: BufFromGl,
    }
    println!("{} gen_textures", std::mem::size_of::<gen_textures>());

    struct gen_vertex_arrays {
        n: GLsizei,
        returned: BufFromGl,
    }
    println!(
        "{} gen_vertex_arrays",
        std::mem::size_of::<gen_vertex_arrays>()
    );

    struct gen_vertex_arrays_apple {
        n: GLsizei,
        returned: BufFromGl,
    }
    println!(
        "{} gen_vertex_arrays_apple",
        std::mem::size_of::<gen_vertex_arrays_apple>()
    );

    struct line_width {
        width: GLfloat,
    }
    println!("{} line_width", std::mem::size_of::<line_width>());

    struct pixel_store_i {
        name: GLenum,
        param: GLint,
    }
    println!("{} pixel_store_i", std::mem::size_of::<pixel_store_i>());

    struct scissor {
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
    }
    println!("{} scissor", std::mem::size_of::<scissor>());

    struct tex_image_2d {
        target: GLenum,
        level: GLint,
        internal_format: GLint,
        width: GLsizei,
        height: GLsizei,
        border: GLint,
        format: GLenum,
        ty: GLenum,
        opt_data: Option<BufToGl>,
    }
    println!("{} tex_image_2d", std::mem::size_of::<tex_image_2d>());

    struct tex_image_3d {
        target: GLenum,
        level: GLint,
        internal_format: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        border: GLint,
        format: GLenum,
        ty: GLenum,
        opt_data: Option<BufToGl>,
    }
    println!("{} tex_image_3d", std::mem::size_of::<tex_image_3d>());

    struct tex_parameter_f {
        target: GLenum,
        pname: GLenum,
        param: GLfloat,
    }
    println!("{} tex_parameter_f", std::mem::size_of::<tex_parameter_f>());

    struct tex_parameter_i {
        target: GLenum,
        pname: GLenum,
        param: GLint,
    }
    println!("{} tex_parameter_i", std::mem::size_of::<tex_parameter_i>());

    struct tex_sub_image_3d {
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        zoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        format: GLenum,
        ty: GLenum,
        data: BufToGl,
    }
    println!(
        "{} tex_sub_image_3d",
        std::mem::size_of::<tex_sub_image_3d>()
    );

    struct use_program {
        program: GLuint,
    }
    println!("{} use_program", std::mem::size_of::<use_program>());

    struct vertex_attrib_divisor {
        index: GLuint,
        divisor: GLuint,
    }
    println!(
        "{} vertex_attrib_divisor",
        std::mem::size_of::<vertex_attrib_divisor>()
    );

    struct vertex_attrib_i_pointer {
        index: GLuint,
        size: GLint,
        type_: GLenum,
        stride: GLsizei,
        offset: GLuint,
    }
    println!(
        "{} vertex_attrib_i_pointer",
        std::mem::size_of::<vertex_attrib_i_pointer>()
    );

    struct vertex_attrib_pointer {
        index: GLuint,
        size: GLint,
        type_: GLenum,
        normalized: bool,
        stride: GLsizei,
        offset: GLuint,
    }
    println!(
        "{} vertex_attrib_pointer",
        std::mem::size_of::<vertex_attrib_pointer>()
    );

    struct viewport {
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
    }
    println!("{} viewport", std::mem::size_of::<viewport>());

    struct bind_vertex_array_apple {
        vao: GLuint,
    }
    println!(
        "{} bind_vertex_array_apple",
        std::mem::size_of::<bind_vertex_array_apple>()
    );

    struct bind_renderbuffer {
        target: GLenum,
        renderbuffer: GLuint,
    }
    println!(
        "{} bind_renderbuffer",
        std::mem::size_of::<bind_renderbuffer>()
    );

    struct bind_framebuffer {
        target: GLenum,
        framebuffer: GLuint,
    }
    println!(
        "{} bind_framebuffer",
        std::mem::size_of::<bind_framebuffer>()
    );

    struct framebuffer_texture_2d {
        target: GLenum,
        attachment: GLenum,
        textarget: GLenum,
        texture: GLuint,
        level: GLint,
    }
    println!(
        "{} framebuffer_texture_2d",
        std::mem::size_of::<framebuffer_texture_2d>()
    );

    struct framebuffer_texture_layer {
        target: GLenum,
        attachment: GLenum,
        texture: GLuint,
        level: GLint,
        layer: GLint,
    }
    println!(
        "{} framebuffer_texture_layer",
        std::mem::size_of::<framebuffer_texture_layer>()
    );

    struct blit_framebuffer {
        src_x0: GLint,
        src_y0: GLint,
        src_x1: GLint,
        src_y1: GLint,
        dst_x0: GLint,
        dst_y0: GLint,
        dst_x1: GLint,
        dst_y1: GLint,
        mask: GLbitfield,
        filter: GLenum,
    }
    println!(
        "{} blit_framebuffer",
        std::mem::size_of::<blit_framebuffer>()
    );

    struct hint {
        param_name: GLenum,
        param_val: GLenum,
    }
    println!("{} hint", std::mem::size_of::<hint>());

    struct is_enabled {
        cap: GLenum,
    }
    println!("{} is_enabled", std::mem::size_of::<is_enabled>());

    struct is_shader {
        shader: GLuint,
    }
    println!("{} is_shader", std::mem::size_of::<is_shader>());

    struct is_texture {
        texture: GLenum,
    }
    println!("{} is_texture", std::mem::size_of::<is_texture>());

    struct is_framebuffer {
        framebuffer: GLenum,
    }
    println!("{} is_framebuffer", std::mem::size_of::<is_framebuffer>());

    struct is_renderbuffer {
        renderbuffer: GLenum,
    }
    println!("{} is_renderbuffer", std::mem::size_of::<is_renderbuffer>());

    struct check_frame_buffer_status {
        target: GLenum,
    }
    println!(
        "{} check_frame_buffer_status",
        std::mem::size_of::<check_frame_buffer_status>()
    );

    struct renderbuffer_storage {
        target: GLenum,
        internalformat: GLenum,
        width: GLsizei,
        height: GLsizei,
    }
    println!(
        "{} renderbuffer_storage",
        std::mem::size_of::<renderbuffer_storage>()
    );

    struct framebuffer_renderbuffer {
        target: GLenum,
        attachment: GLenum,
        renderbuffertarget: GLenum,
        renderbuffer: GLuint,
    }
    println!(
        "{} framebuffer_renderbuffer",
        std::mem::size_of::<framebuffer_renderbuffer>()
    );

    struct tex_sub_image_2d_pbo {
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        ty: GLenum,
        offset: usize,
    }
    println!(
        "{} tex_sub_image_2d_pbo",
        std::mem::size_of::<tex_sub_image_2d_pbo>()
    );

    struct flush {}
    println!("{} flush", std::mem::size_of::<flush>());

    struct finish {}
    println!("{} finish", std::mem::size_of::<finish>());

    struct depth_mask {
        flag: bool,
    }
    println!("{} depth_mask", std::mem::size_of::<depth_mask>());

    struct create_program {
        returned: GLuint,
    }
    println!("{} create_program", std::mem::size_of::<create_program>());

    struct create_shader {
        shader_type: GLenum,
        returned: GLuint,
    }
    println!("{} create_shader", std::mem::size_of::<create_shader>());

    struct shader_source {
        shader: GLuint,
        strings: BufToGl,
    }
    println!("{} shader_source", std::mem::size_of::<shader_source>());

    struct compile_shader {
        shader: GLuint,
    }
    println!("{} compile_shader", std::mem::size_of::<compile_shader>());

    struct get_shader_iv {
        shader: GLuint,
        pname: GLenum,
        result: BufFromGl,
    }
    println!("{} get_shader_iv", std::mem::size_of::<get_shader_iv>());

    struct attach_shader {
        program: GLuint,
        shader: GLuint,
    }
    println!("{} attach_shader", std::mem::size_of::<attach_shader>());

    struct bind_attrib_location {
        program: GLuint,
        index: GLuint,
        name: BufToGl,
    }
    println!(
        "{} bind_attrib_location",
        std::mem::size_of::<bind_attrib_location>()
    );

    struct link_program {
        program: GLuint,
    }
    println!("{} link_program", std::mem::size_of::<link_program>());

    struct delete_shader {
        shader: GLuint,
    }
    println!("{} delete_shader", std::mem::size_of::<delete_shader>());

    struct detach_shader {
        program: GLuint,
        shader: GLuint,
    }
    println!("{} detach_shader", std::mem::size_of::<detach_shader>());

    struct clear {
        buffer_mask: GLbitfield,
    }
    println!("{} clear", std::mem::size_of::<clear>());

    struct clear_depth {
        depth: f64,
    }
    println!("{} clear_depth", std::mem::size_of::<clear_depth>());

    struct clear_stencil {
        s: GLint,
    }
    println!("{} clear_stencil", std::mem::size_of::<clear_stencil>());

    struct get_attrib_location {
        program: GLuint,
        name: BufToGl,
    }
    println!(
        "{} get_attrib_location",
        std::mem::size_of::<get_attrib_location>()
    );

    struct get_frag_data_location {
        program: GLuint,
        name: BufToGl,
    }
    println!(
        "{} get_frag_data_location",
        std::mem::size_of::<get_frag_data_location>()
    );

    struct get_uniform_location {
        program: GLuint,
        name: BufToGl,
    }
    println!(
        "{} get_uniform_location",
        std::mem::size_of::<get_uniform_location>()
    );

    struct get_program_iv {
        program: GLuint,
        pname: GLenum,
        result: BufFromGl,
    }
    println!("{} get_program_iv", std::mem::size_of::<get_program_iv>());

    struct uniform_1i {
        location: GLint,
        v0: GLint,
    }
    println!("{} uniform_1i", std::mem::size_of::<uniform_1i>());

    struct uniform_1iv {
        location: GLint,
        values: BufToGl,
    }
    println!("{} uniform_1iv", std::mem::size_of::<uniform_1iv>());

    struct uniform_1f {
        location: GLint,
        v0: GLfloat,
    }
    println!("{} uniform_1f", std::mem::size_of::<uniform_1f>());

    struct uniform_1fv {
        location: GLint,
        values: BufToGl,
    }
    println!("{} uniform_1fv", std::mem::size_of::<uniform_1fv>());

    struct uniform_1ui {
        location: GLint,
        v0: GLuint,
    }
    println!("{} uniform_1ui", std::mem::size_of::<uniform_1ui>());

    struct uniform_2f {
        location: GLint,
        v0: GLfloat,
        v1: GLfloat,
    }
    println!("{} uniform_2f", std::mem::size_of::<uniform_2f>());

    struct uniform_2fv {
        location: GLint,
        values: BufToGl,
    }
    println!("{} uniform_2fv", std::mem::size_of::<uniform_2fv>());

    struct uniform_2i {
        location: GLint,
        v0: GLint,
        v1: GLint,
    }
    println!("{} uniform_2i", std::mem::size_of::<uniform_2i>());

    struct uniform_2iv {
        location: GLint,
        values: BufToGl,
    }
    println!("{} uniform_2iv", std::mem::size_of::<uniform_2iv>());

    struct uniform_2ui {
        location: GLint,
        v0: GLuint,
        v1: GLuint,
    }
    println!("{} uniform_2ui", std::mem::size_of::<uniform_2ui>());

    struct uniform_3f {
        location: GLint,
        v0: GLfloat,
        v1: GLfloat,
        v2: GLfloat,
    }
    println!("{} uniform_3f", std::mem::size_of::<uniform_3f>());

    struct uniform_3fv {
        location: GLint,
        values: BufToGl,
    }
    println!("{} uniform_3fv", std::mem::size_of::<uniform_3fv>());

    struct uniform_3i {
        location: GLint,
        v0: GLint,
        v1: GLint,
        v2: GLint,
    }
    println!("{} uniform_3i", std::mem::size_of::<uniform_3i>());

    struct uniform_3iv {
        location: GLint,
        values: BufToGl,
    }
    println!("{} uniform_3iv", std::mem::size_of::<uniform_3iv>());

    struct uniform_3ui {
        location: GLint,
        v0: GLuint,
        v1: GLuint,
        v2: GLuint,
    }
    println!("{} uniform_3ui", std::mem::size_of::<uniform_3ui>());

    struct uniform_4f {
        location: GLint,
        x: GLfloat,
        y: GLfloat,
        z: GLfloat,
        w: GLfloat,
    }
    println!("{} uniform_4f", std::mem::size_of::<uniform_4f>());

    struct uniform_4i {
        location: GLint,
        x: GLint,
        y: GLint,
        z: GLint,
        w: GLint,
    }
    println!("{} uniform_4i", std::mem::size_of::<uniform_4i>());

    struct uniform_4iv {
        location: GLint,
        values: BufToGl,
    }
    println!("{} uniform_4iv", std::mem::size_of::<uniform_4iv>());

    struct uniform_4ui {
        location: GLint,
        x: GLuint,
        y: GLuint,
        z: GLuint,
        w: GLuint,
    }
    println!("{} uniform_4ui", std::mem::size_of::<uniform_4ui>());

    struct uniform_4fv {
        location: GLint,
        values: BufToGl,
    }
    println!("{} uniform_4fv", std::mem::size_of::<uniform_4fv>());

    struct uniform_matrix_2fv {
        location: GLint,
        transpose: bool,
        value: BufToGl,
    }
    println!(
        "{} uniform_matrix_2fv",
        std::mem::size_of::<uniform_matrix_2fv>()
    );

    struct uniform_matrix_3fv {
        location: GLint,
        transpose: bool,
        value: BufToGl,
    }
    println!(
        "{} uniform_matrix_3fv",
        std::mem::size_of::<uniform_matrix_3fv>()
    );

    struct uniform_matrix_4fv {
        location: GLint,
        transpose: bool,
        value: BufToGl,
    }
    println!(
        "{} uniform_matrix_4fv",
        std::mem::size_of::<uniform_matrix_4fv>()
    );

    struct depth_range {
        near: f64,
        far: f64,
    }
    println!("{} depth_range", std::mem::size_of::<depth_range>());

    struct draw_elements_instanced {
        mode: GLenum,
        count: GLsizei,
        element_type: GLenum,
        indices_offset: GLuint,
        primcount: GLsizei,
    }
    println!(
        "{} draw_elements_instanced",
        std::mem::size_of::<draw_elements_instanced>()
    );

    struct blend_color {
        r: f32,
        g: f32,
        b: f32,
        a: f32,
    }
    println!("{} blend_color", std::mem::size_of::<blend_color>());

    struct blend_func {
        sfactor: GLenum,
        dfactor: GLenum,
    }
    println!("{} blend_func", std::mem::size_of::<blend_func>());

    struct blend_func_separate {
        src_rgb: GLenum,
        dest_rgb: GLenum,
        src_alpha: GLenum,
        dest_alpha: GLenum,
    }
    println!(
        "{} blend_func_separate",
        std::mem::size_of::<blend_func_separate>()
    );

    struct blend_equation {
        mode: GLenum,
    }
    println!("{} blend_equation", std::mem::size_of::<blend_equation>());

    struct blend_equation_separate {
        mode_rgb: GLenum,
        mode_alpha: GLenum,
    }
    println!(
        "{} blend_equation_separate",
        std::mem::size_of::<blend_equation_separate>()
    );

    struct color_mask {
        r: bool,
        g: bool,
        b: bool,
        a: bool,
    }
    println!("{} color_mask", std::mem::size_of::<color_mask>());

    struct cull_face {
        mode: GLenum,
    }
    println!("{} cull_face", std::mem::size_of::<cull_face>());

    struct front_face {
        mode: GLenum,
    }
    println!("{} front_face", std::mem::size_of::<front_face>());

    struct depth_func {
        func: GLenum,
    }
    println!("{} depth_func", std::mem::size_of::<depth_func>());

    struct invalidate_framebuffer {
        target: GLenum,
        attachments: BufToGl,
    }
    println!(
        "{} invalidate_framebuffer",
        std::mem::size_of::<invalidate_framebuffer>()
    );

    struct invalidate_sub_framebuffer {
        target: GLenum,
        attachments: BufToGl,
        xoffset: GLint,
        yoffset: GLint,
        width: GLsizei,
        height: GLsizei,
    }
    println!(
        "{} invalidate_sub_framebuffer",
        std::mem::size_of::<invalidate_sub_framebuffer>()
    );

    struct read_buffer {
        mode: GLenum,
    }
    println!("{} read_buffer", std::mem::size_of::<read_buffer>());

    struct read_pixels_into_buffer {
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        pixel_type: GLenum,
        dst_buffer: BufFromGl,
    }
    println!(
        "{} read_pixels_into_buffer",
        std::mem::size_of::<read_pixels_into_buffer>()
    );

    struct read_pixels {
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        pixel_type: GLenum,
    }
    println!("{} read_pixels", std::mem::size_of::<read_pixels>());

    struct read_pixels_into_pbo {
        x: GLint,
        y: GLint,
        width: GLsizei,
        height: GLsizei,
        format: GLenum,
        pixel_type: GLenum,
    }
    println!(
        "{} read_pixels_into_pbo",
        std::mem::size_of::<read_pixels_into_pbo>()
    );

    struct sample_coverage {
        value: GLclampf,
        invert: bool,
    }
    println!("{} sample_coverage", std::mem::size_of::<sample_coverage>());

    struct polygon_offset {
        factor: GLfloat,
        units: GLfloat,
    }
    println!("{} polygon_offset", std::mem::size_of::<polygon_offset>());

    struct begin_query {
        target: GLenum,
        id: GLuint,
    }
    println!("{} begin_query", std::mem::size_of::<begin_query>());

    struct end_query {
        target: GLenum,
    }
    println!("{} end_query", std::mem::size_of::<end_query>());

    struct query_counter {
        id: GLuint,
        target: GLenum,
    }
    println!("{} query_counter", std::mem::size_of::<query_counter>());

    struct get_query_object_iv {
        id: GLuint,
        pname: GLenum,
        returned: i32,
    }
    println!(
        "{} get_query_object_iv",
        std::mem::size_of::<get_query_object_iv>()
    );

    struct get_query_object_uiv {
        id: GLuint,
        pname: GLenum,
        returned: u32,
    }
    println!(
        "{} get_query_object_uiv",
        std::mem::size_of::<get_query_object_uiv>()
    );

    struct get_query_object_i64v {
        id: GLuint,
        pname: GLenum,
        returned: i64,
    }
    println!(
        "{} get_query_object_i64v",
        std::mem::size_of::<get_query_object_i64v>()
    );

    struct get_query_object_ui64v {
        id: GLuint,
        pname: GLenum,
        returned: u64,
    }
    println!(
        "{} get_query_object_ui64v",
        std::mem::size_of::<get_query_object_ui64v>()
    );

    struct delete_queries {
        queries: BufToGl,
    }
    println!("{} delete_queries", std::mem::size_of::<delete_queries>());

    struct delete_vertex_arrays {
        vertex_arrays: BufToGl,
    }
    println!(
        "{} delete_vertex_arrays",
        std::mem::size_of::<delete_vertex_arrays>()
    );

    struct delete_vertex_arrays_apple {
        vertex_arrays: BufToGl,
    }
    println!(
        "{} delete_vertex_arrays_apple",
        std::mem::size_of::<delete_vertex_arrays_apple>()
    );

    struct delete_buffers {
        buffers: BufToGl,
    }
    println!("{} delete_buffers", std::mem::size_of::<delete_buffers>());

    struct delete_renderbuffers {
        renderbuffers: BufToGl,
    }
    println!(
        "{} delete_renderbuffers",
        std::mem::size_of::<delete_renderbuffers>()
    );

    struct delete_framebuffers {
        framebuffers: BufToGl,
    }
    println!(
        "{} delete_framebuffers",
        std::mem::size_of::<delete_framebuffers>()
    );

    struct delete_textures {
        textures: BufToGl,
    }
    println!("{} delete_textures", std::mem::size_of::<delete_textures>());

    struct delete_program {
        program: GLuint,
    }
    println!("{} delete_program", std::mem::size_of::<delete_program>());

    struct tex_sub_image_3d_pbo {
        target: GLenum,
        level: GLint,
        xoffset: GLint,
        yoffset: GLint,
        zoffset: GLint,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
        format: GLenum,
        ty: GLenum,
        offset: usize,
    }
    println!(
        "{} tex_sub_image_3d_pbo",
        std::mem::size_of::<tex_sub_image_3d_pbo>()
    );

    struct tex_storage_2d {
        target: GLenum,
        levels: GLint,
        internal_format: GLenum,
        width: GLsizei,
        height: GLsizei,
    }
    println!("{} tex_storage_2d", std::mem::size_of::<tex_storage_2d>());

    struct tex_storage_3d {
        target: GLenum,
        levels: GLint,
        internal_format: GLenum,
        width: GLsizei,
        height: GLsizei,
        depth: GLsizei,
    }
    println!("{} tex_storage_3d", std::mem::size_of::<tex_storage_3d>());

    struct get_tex_image_into_buffer {
        target: GLenum,
        level: GLint,
        format: GLenum,
        ty: GLenum,
        output: BufFromGl,
    }
    println!(
        "{} get_tex_image_into_buffer",
        std::mem::size_of::<get_tex_image_into_buffer>()
    );

    struct copy_image_sub_data {
        src_name: GLuint,
        src_target: GLenum,
        src_level: GLint,
        src_x: GLint,
        src_y: GLint,
        src_z: GLint,
        dst_name: GLuint,
        dst_target: GLenum,
        dst_level: GLint,
        dst_x: GLint,
        dst_y: GLint,
        dst_z: GLint,
        src_width: GLsizei,
        src_height: GLsizei,
        src_depth: GLsizei,
    }
    println!(
        "{} copy_image_sub_data",
        std::mem::size_of::<copy_image_sub_data>()
    );

    struct generate_mipmap {
        target: GLenum,
    }
    println!("{} generate_mipmap", std::mem::size_of::<generate_mipmap>());
}
