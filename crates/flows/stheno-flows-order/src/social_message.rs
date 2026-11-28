//! `Ref` codes and the buyer-facing message for social handoffs (PRODUCT.md §5.3).
//!
//! Neither existed. The storefront's social-checkout component sent a one-line
//! "Hi! I would like to order …" with no reference at all — and `Ref` is **the only link
//! between a chat conversation and the queued draft order**. Without it staff cannot
//! match an incoming message to a dashboard row, which is the entire point of the social
//! queue.

use rand::Rng;

/// Alphabet for `Ref` codes.
///
/// Characters confused when read aloud or over a phone are excluded: no `0`/`O`, no
/// `1`/`I`. This is not cosmetic — staff hearing "oh" and typing zero loses the order.
const REF_ALPHABET: &[u8] = b"23456789ABCDEFGHJKLMNPQRSTUVWXYZ";

/// Short enough to say out loud, long enough that collisions stay negligible at
/// realistic social-order volumes (32^6 ≈ 1.07 billion). The fixed VX prefix identifies
/// the reference as a Vendryx social order when it is copied between channels.
const REF_LENGTH: usize = 6;

/// Generates a short, unambiguous reference code for a social draft.
pub fn generate_ref_code() -> String {
    let mut rng = rand::thread_rng();
    let suffix: String = (0..REF_LENGTH)
        .map(|_| REF_ALPHABET[rng.gen_range(0..REF_ALPHABET.len())] as char)
        .collect();
    format!("VX-{suffix}")
}

/// One line of a social order message.
#[derive(Debug, Clone)]
pub struct SocialMessageLine {
    pub title: String,
    pub sku: Option<String>,
    pub options: Vec<String>,
    pub quantity: i32,
    pub unit_price: String,
    pub line_total: String,
}

/// Renders the buyer-facing message for a social handoff.
///
/// The shape matters: staff match an incoming chat message to a dashboard row by eye, on
/// a phone, so the product, its code, the quantity and the `Ref` all have to be legible
/// in a chat bubble.
pub fn render_social_message(
    store_name: &str,
    lines: &[SocialMessageLine],
    total: &str,
    ref_code: &str,
) -> String {
    let rule = "\u{2500}".repeat(5);
    let mut out = format!("Order from {store_name}\n{rule}\n");

    for (index, line) in lines.iter().enumerate() {
        out.push_str(&format!("{}. {}\n", index + 1, line.title));
        if let Some(sku) = &line.sku {
            out.push_str(&format!("   Code: {sku}\n"));
        }
        if !line.options.is_empty() {
            out.push_str(&format!("   {}\n", line.options.join(" \u{b7} ")));
        }
        out.push_str(&format!(
            "   Qty {} \u{d7} {} = {}\n",
            line.quantity, line.unit_price, line.line_total
        ));
    }

    out.push_str(&format!("{rule}\nTotal: {total}\nRef: #{ref_code}"));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ref_codes_never_contain_characters_confused_when_read_aloud() {
        // The rule this alphabet exists for. Generated over many samples because a
        // single draw proves nothing about a random generator.
        for _ in 0..2_000 {
            let code = generate_ref_code();
            assert!(code.starts_with("VX-"));
            let suffix = &code[3..];
            assert_eq!(suffix.len(), REF_LENGTH);
            for c in suffix.chars() {
                assert!(
                    !matches!(c, '0' | 'O' | '1' | 'I'),
                    "{code} contains an ambiguous character"
                );
                assert!(c.is_ascii_uppercase() || c.is_ascii_digit());
            }
        }
    }

    #[test]
    fn ref_codes_are_not_all_the_same() {
        let a: std::collections::HashSet<String> = (0..200).map(|_| generate_ref_code()).collect();
        assert!(a.len() > 190, "generator is not producing distinct codes");
    }

    #[test]
    fn the_message_carries_everything_staff_need_to_match_it() {
        let message = render_social_message(
            "Acme",
            &[SocialMessageLine {
                title: "Merino Crew".to_string(),
                sku: Some("MC-BLU-M".to_string()),
                options: vec!["Colour: Blue".to_string(), "Size: M".to_string()],
                quantity: 2,
                unit_price: "$40.00".to_string(),
                line_total: "$80.00".to_string(),
            }],
            "$80.00",
            "VX-K7M2QP",
        );

        assert!(message.starts_with("Order from Acme"));
        assert!(message.contains("1. Merino Crew"));
        assert!(message.contains("Code: MC-BLU-M"));
        assert!(message.contains("Colour: Blue"));
        assert!(message.contains("Qty 2"));
        assert!(message.contains("Total: $80.00"));
        // Without this line the draft cannot be matched to the conversation at all.
        assert!(message.ends_with("Ref: #VX-K7M2QP"));
    }

    #[test]
    fn a_line_without_a_sku_omits_the_code_row_rather_than_printing_empty() {
        let message = render_social_message(
            "Acme",
            &[SocialMessageLine {
                title: "Gift Card".to_string(),
                sku: None,
                options: vec![],
                quantity: 1,
                unit_price: "$25.00".to_string(),
                line_total: "$25.00".to_string(),
            }],
            "$25.00",
            "VX-ABC234",
        );
        assert!(!message.contains("Code:"));
    }
}
