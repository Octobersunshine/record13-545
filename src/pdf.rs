use crate::models::{MedicalRecord, Pet};
use printpdf::*;
use std::io::BufWriter;

const FONT_SIZE_TITLE: f32 = 18.0;
const FONT_SIZE_SUBTITLE: f32 = 14.0;
const FONT_SIZE_BODY: f32 = 11.0;
const FONT_SIZE_SMALL: f32 = 9.0;
const MARGIN_LEFT_MM: f32 = 20.0;
const MARGIN_TOP_MM: f32 = 270.0;
const LINE_HEIGHT_MM: f32 = 7.0;
const PAGE_WIDTH_MM: f32 = 210.0;
const PAGE_HEIGHT_MM: f32 = 297.0;
const MAX_CHARS_PER_LINE: usize = 48;

pub struct MedicalRecordPdfData {
    pub pet: Pet,
    pub record: MedicalRecord,
}

pub fn generate_medical_record_pdf(data: &MedicalRecordPdfData) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
    let doc = PdfDocument::new(
        "Pet Medical Record",
        Mm(PAGE_WIDTH_MM),
        Mm(PAGE_HEIGHT_MM),
        "Layer 1",
    );
    let page = doc.get_page(0);
    let layer = page.get_layer(0);

    let font = doc.add_builtin_font(BuiltinFont::Helvetica)?;
    let font_bold = doc.add_builtin_font(BuiltinFont::HelveticaBold)?;

    let mut y = MARGIN_TOP_MM;

    y = draw_title(&layer, &font_bold, &data.record, y);
    y = draw_divider(&layer, y);
    y = draw_pet_info(&layer, &font, &font_bold, &data.pet, y);
    y = draw_divider(&layer, y);
    y = draw_medical_info(&layer, &font, &font_bold, &data.record, y);
    y = draw_divider(&layer, y);
    draw_footer(&layer, &font, y);

    let mut buf = BufWriter::new(Vec::new());
    doc.save(&mut buf)?;
    Ok(buf.into_inner()?)
}

fn draw_title(layer: &PdfLayerReference, font: &IndirectFontRef, record: &MedicalRecord, y: f32) -> f32 {
    layer.use_text("Pet Medical Record", FONT_SIZE_TITLE, Mm(MARGIN_LEFT_MM), Mm(y), font);

    let record_id = record.id.to_string();
    let short_id = if record_id.len() > 8 {
        &record_id[..8]
    } else {
        &record_id
    };
    let sub = format!("ID: {}", short_id);
    layer.use_text(sub, FONT_SIZE_SMALL, Mm(130.0), Mm(y), font);

    y - LINE_HEIGHT_MM * 2.0
}

fn draw_divider(layer: &PdfLayerReference, y: f32) -> f32 {
    layer.set_line_width(0.3);
    layer.set_outline_color(&Color::Rgb(Rgb::new(0.75, 0.75, 0.75, None)));
    layer.add_line(Line::new(vec![
        (Point::new(Mm(MARGIN_LEFT_MM), Mm(y)), false),
        (Point::new(Mm(PAGE_WIDTH_MM - 20.0), Mm(y)), false),
    ]));

    y - LINE_HEIGHT_MM * 0.5
}

fn draw_pet_info(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    pet: &Pet,
    y: f32,
) -> f32 {
    let mut cy = y;

    layer.use_text("Pet Information", FONT_SIZE_SUBTITLE, Mm(MARGIN_LEFT_MM), Mm(cy), font_bold);
    cy -= LINE_HEIGHT_MM * 1.2;

    let breed = pet.breed.as_deref().unwrap_or("N/A");
    let age = pet.age.map(|a| a.to_string()).unwrap_or_else(|| "N/A".to_string());

    let lines = vec![
        format!("Name: {}    Species: {}    Breed: {}", pet.name, pet.species, breed),
        format!("Age: {}    Owner: {}    Phone: {}", age, pet.owner_name, pet.owner_phone),
    ];

    for line in &lines {
        layer.use_text(line, FONT_SIZE_BODY, Mm(MARGIN_LEFT_MM), Mm(cy), font);
        cy -= LINE_HEIGHT_MM;
    }

    cy
}

fn draw_medical_info(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    record: &MedicalRecord,
    y: f32,
) -> f32 {
    let mut cy = y;

    layer.use_text("Medical Record", FONT_SIZE_SUBTITLE, Mm(MARGIN_LEFT_MM), Mm(cy), font_bold);
    cy -= LINE_HEIGHT_MM * 1.2;

    let visit_date = record.visit_date.format("%Y-%m-%d %H:%M UTC").to_string();
    layer.use_text(&format!("Visit Date: {}", visit_date), FONT_SIZE_BODY, Mm(MARGIN_LEFT_MM), Mm(cy), font);
    cy -= LINE_HEIGHT_MM;

    layer.use_text(&format!("Veterinarian: {}", record.veterinarian), FONT_SIZE_BODY, Mm(MARGIN_LEFT_MM), Mm(cy), font);
    cy -= LINE_HEIGHT_MM * 1.5;

    cy = draw_section(layer, font, font_bold, "Diagnosis", &record.diagnosis, cy);
    cy = draw_section(layer, font, font_bold, "Treatment", &record.treatment, cy);

    if let Some(ref prescription) = record.prescription {
        cy = draw_section(layer, font, font_bold, "Prescription", prescription, cy);
    }

    if let Some(ref notes) = record.notes {
        cy = draw_section(layer, font, font_bold, "Notes", notes, cy);
    }

    cy
}

fn draw_section(
    layer: &PdfLayerReference,
    font: &IndirectFontRef,
    font_bold: &IndirectFontRef,
    label: &str,
    content: &str,
    y: f32,
) -> f32 {
    let mut cy = y;

    layer.use_text(label, FONT_SIZE_BODY, Mm(MARGIN_LEFT_MM), Mm(cy), font_bold);
    cy -= LINE_HEIGHT_MM;

    for line in wrap_text(content, MAX_CHARS_PER_LINE) {
        layer.use_text(&line, FONT_SIZE_BODY, Mm(MARGIN_LEFT_MM + 5.0), Mm(cy), font);
        cy -= LINE_HEIGHT_MM;
    }

    cy - LINE_HEIGHT_MM * 0.3
}

fn wrap_text(text: &str, max_chars: usize) -> Vec<String> {
    let mut result = Vec::new();
    let mut current = String::new();
    let mut char_count = 0;

    for ch in text.chars() {
        if ch == '\n' {
            result.push(current.clone());
            current.clear();
            char_count = 0;
            continue;
        }

        let width = if ch.is_ascii() { 1 } else { 2 };
        if char_count + width > max_chars && !current.is_empty() {
            result.push(current.clone());
            current.clear();
            char_count = 0;
        }

        current.push(ch);
        char_count += width;
    }

    if !current.is_empty() {
        result.push(current);
    }

    if result.is_empty() {
        result.push(String::new());
    }

    result
}

fn draw_footer(layer: &PdfLayerReference, font: &IndirectFontRef, y: f32) -> f32 {
    let now = chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
    layer.use_text(&format!("Generated: {}", now), FONT_SIZE_SMALL, Mm(MARGIN_LEFT_MM), Mm(y), font);

    layer.use_text(
        "This document is auto-generated by Pet Medical Record System",
        FONT_SIZE_SMALL,
        Mm(MARGIN_LEFT_MM),
        Mm(y - LINE_HEIGHT_MM),
        font,
    );

    y - LINE_HEIGHT_MM * 2.0
}
