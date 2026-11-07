use crate::{shapes::spline::sink::SplineSink, Spline, V2};

#[derive(Debug, Clone, Copy)]
enum SvgPathToken {
    Command(char),
    Number(f32),
}

/// Parses SVG path data into one exact `Spline` per subpath, in the data's own units and orientation.
pub(crate) fn parse_svg_path_data(path_data: &str) -> Vec<Spline> {
    let mut sink = SplineSink::new();
    let mut last_cubic_control: Option<V2> = None;
    let mut last_quad_control: Option<V2> = None;

    let mut active_command = 'M';
    let mut command_is_set = false;

    let tokens = tokenize_svg_path_data(path_data);
    let mut token_index = 0;

    while token_index < tokens.len() {
        if let SvgPathToken::Command(command) = tokens[token_index] {
            active_command = command;
            command_is_set = true;
            token_index += 1;
        } else if !command_is_set {
            // TODO: Handle malformed path data where numbers appear before the first command.
            break;
        }

        let relative = active_command.is_ascii_lowercase();
        let offset = |sink: &SplineSink| {
            if relative {
                sink.current()
            } else {
                V2::zero()
            }
        };

        match active_command {
            'M' | 'm' => {
                let mut is_first_pair = true;
                while let Some((x, y)) = read_svg_pair(&tokens, &mut token_index) {
                    let next_point = offset(&sink) + V2::new(x, y);
                    if is_first_pair {
                        sink.move_to(next_point);
                        is_first_pair = false;
                    } else {
                        sink.line_to(next_point);
                    }
                }
            }
            'L' | 'l' => {
                while let Some((x, y)) = read_svg_pair(&tokens, &mut token_index) {
                    sink.line_to(offset(&sink) + V2::new(x, y));
                }
            }
            'H' | 'h' => {
                while let Some(x) = read_svg_number(&tokens, &mut token_index) {
                    let base = sink.current();
                    sink.line_to(V2::new(offset(&sink).x + x, base.y));
                }
            }
            'V' | 'v' => {
                while let Some(y) = read_svg_number(&tokens, &mut token_index) {
                    let base = sink.current();
                    sink.line_to(V2::new(base.x, offset(&sink).y + y));
                }
            }
            'Z' | 'z' => sink.close(),
            'C' | 'c' => {
                while let (Some(x1), Some(y1), Some(x2), Some(y2), Some(x), Some(y)) = (
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                ) {
                    let offset = offset(&sink);
                    let ctrl1 = offset + V2::new(x1, y1);
                    let ctrl2 = offset + V2::new(x2, y2);
                    sink.cubic_to(ctrl1, ctrl2, offset + V2::new(x, y));
                    last_cubic_control = Some(ctrl2);
                }
            }
            'S' | 's' => {
                while let (Some(x2), Some(y2), Some(x), Some(y)) = (
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                ) {
                    let base = sink.current();
                    let offset = offset(&sink);
                    let ctrl1 = last_cubic_control.map_or(base, |ctrl| base * 2.0 - ctrl);
                    let ctrl2 = offset + V2::new(x2, y2);
                    sink.cubic_to(ctrl1, ctrl2, offset + V2::new(x, y));
                    last_cubic_control = Some(ctrl2);
                }
            }
            'Q' | 'q' => {
                while let (Some(cx), Some(cy), Some(x), Some(y)) = (
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                ) {
                    let offset = offset(&sink);
                    let ctrl = offset + V2::new(cx, cy);
                    sink.quad_to(ctrl, offset + V2::new(x, y));
                    last_quad_control = Some(ctrl);
                }
            }
            'T' | 't' => {
                while let Some((x, y)) = read_svg_pair(&tokens, &mut token_index) {
                    let base = sink.current();
                    let ctrl = last_quad_control.map_or(base, |ctrl| base * 2.0 - ctrl);
                    sink.quad_to(ctrl, offset(&sink) + V2::new(x, y));
                    last_quad_control = Some(ctrl);
                }
            }
            'A' | 'a' => {
                while let (
                    Some(_rx),
                    Some(_ry),
                    Some(_x_axis_rotation),
                    Some(_large_arc_flag),
                    Some(_sweep_flag),
                    Some(x),
                    Some(y),
                ) = (
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                    read_svg_number(&tokens, &mut token_index),
                ) {
                    // TODO: Sample elliptical arcs instead of endpoint approximation.
                    sink.line_to(offset(&sink) + V2::new(x, y));
                }
            }
            _ => {
                // TODO: Support additional SVG path commands.
                while read_svg_number(&tokens, &mut token_index).is_some() {}
            }
        }
        if !matches!(active_command, 'C' | 'c' | 'S' | 's') {
            last_cubic_control = None;
        }
        if !matches!(active_command, 'Q' | 'q' | 'T' | 't') {
            last_quad_control = None;
        }
    }

    sink.finish()
}

fn tokenize_svg_path_data(path_data: &str) -> Vec<SvgPathToken> {
    let mut tokens = Vec::new();
    let mut chars = path_data.chars().peekable();

    while let Some(ch) = chars.peek().copied() {
        if ch.is_ascii_alphabetic() {
            chars.next();
            tokens.push(SvgPathToken::Command(ch));
            continue;
        }

        if ch.is_ascii_whitespace() || ch == ',' {
            chars.next();
            continue;
        }

        let mut number = String::new();
        let mut seen_exponent = false;

        while let Some(next_ch) = chars.peek().copied() {
            if next_ch.is_ascii_digit() || next_ch == '.' {
                number.push(next_ch);
                chars.next();
                continue;
            }

            if (next_ch == 'e' || next_ch == 'E') && !seen_exponent {
                seen_exponent = true;
                number.push(next_ch);
                chars.next();
                continue;
            }

            if next_ch == '-' || next_ch == '+' {
                if number.is_empty() || number.ends_with('e') || number.ends_with('E') {
                    number.push(next_ch);
                    chars.next();
                    continue;
                }
                break;
            }

            break;
        }

        if !number.is_empty() {
            if let Ok(value) = number.parse::<f32>() {
                tokens.push(SvgPathToken::Number(value));
            } else {
                // TODO: Handle parse failures with richer diagnostics (line/column/path id).
            }
        } else {
            chars.next();
        }
    }

    tokens
}

fn read_svg_number(tokens: &[SvgPathToken], token_index: &mut usize) -> Option<f32> {
    match tokens.get(*token_index)? {
        SvgPathToken::Number(value) => {
            *token_index += 1;
            Some(*value)
        }
        SvgPathToken::Command(_) => None,
    }
}

fn read_svg_pair(tokens: &[SvgPathToken], token_index: &mut usize) -> Option<(f32, f32)> {
    let x = read_svg_number(tokens, token_index)?;
    let y = read_svg_number(tokens, token_index)?;
    Some((x, y))
}
