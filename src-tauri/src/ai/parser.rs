use crate::models::ExtractedLessonItem;

pub fn normalize_hour_string(s: &str) -> Option<String> {
    let clean = s.trim().replace('.', ":").replace('h', ":").replace('H', ":");
    let parts: Vec<&str> = clean.split(':').collect();
    if parts.is_empty() {
        return None;
    }
    let h_digits: String = parts[0].chars().filter(|c| c.is_ascii_digit()).collect();
    if h_digits.is_empty() {
        return None;
    }
    let hours: u32 = h_digits.parse().ok()?;
    if hours > 23 {
        return None;
    }
    let minutes: u32 = if parts.len() > 1 {
        let m_digits: String = parts[1].chars().filter(|c| c.is_ascii_digit()).collect();
        m_digits.parse().unwrap_or(0)
    } else {
        0
    };
    if minutes > 59 {
        return None;
    }
    Some(format!("{:02}:{:02}", hours, minutes))
}

pub fn parse_day_of_week_value(val: &serde_json::Value) -> Option<i32> {
    if let Some(n) = val.as_i64() {
        if (1..=7).contains(&n) {
            return Some(n as i32);
        }
    }
    if let Some(s) = val.as_str() {
        let trimmed = s.trim().to_lowercase();
        if let Ok(n) = trimmed.parse::<i32>() {
            if (1..=7).contains(&n) {
                return Some(n);
            }
        }
        if trimmed.contains("lun") || trimmed.contains("mon") {
            return Some(1);
        }
        if trimmed.contains("mar") || trimmed.contains("tue") {
            return Some(2);
        }
        if trimmed.contains("mer") || trimmed.contains("wed") {
            return Some(3);
        }
        if trimmed.contains("gio") || trimmed.contains("thu") {
            return Some(4);
        }
        if trimmed.contains("ven") || trimmed.contains("fri") {
            return Some(5);
        }
        if trimmed.contains("sab") || trimmed.contains("sat") {
            return Some(6);
        }
        if trimmed.contains("dom") || trimmed.contains("sun") {
            return Some(7);
        }
    }
    None
}

pub fn sanitize_lenient_json(input: &str) -> String {
    let mut out = String::with_capacity(input.len() + 128);
    let chars: Vec<char> = input.chars().collect();
    let n = chars.len();
    let mut i = 0;
    let mut in_double_quote = false;
    let mut in_single_quote = false;
    let mut escape = false;

    while i < n {
        let c = chars[i];
        if in_double_quote {
            out.push(c);
            if escape {
                escape = false;
            } else if c == '\\' {
                escape = true;
            } else if c == '"' {
                in_double_quote = false;
            }
            i += 1;
            continue;
        }
        if in_single_quote {
            if escape {
                out.push(c);
                escape = false;
            } else if c == '\\' {
                out.push(c);
                escape = true;
            } else if c == '\'' {
                out.push('"');
                in_single_quote = false;
            } else if c == '"' {
                out.push_str("\\\"");
            } else {
                out.push(c);
            }
            i += 1;
            continue;
        }
        if c == '"' {
            in_double_quote = true;
            out.push(c);
            i += 1;
            continue;
        }
        if c == '\'' {
            in_single_quote = true;
            out.push('"');
            i += 1;
            continue;
        }
        if c.is_ascii_alphabetic() || c == '_' {
            let start = i;
            while i < n && (chars[i].is_ascii_alphanumeric() || chars[i] == '_') {
                i += 1;
            }
            let word: String = chars[start..i].iter().collect();
            let mut j = i;
            while j < n && (chars[j] == ' ' || chars[j] == '\t' || chars[j] == '\r' || chars[j] == '\n') {
                j += 1;
            }
            if j < n && chars[j] == ':' && word != "true" && word != "false" && word != "null" {
                out.push('"');
                out.push_str(&word);
                out.push('"');
            } else {
                out.push_str(&word);
            }
            continue;
        }
        out.push(c);
        i += 1;
    }
    if in_double_quote || in_single_quote {
        out.push('"');
    }
    out
}

pub fn repair_truncated_json(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.ends_with('}') || trimmed.ends_with(']') {
        return trimmed.to_string();
    }
    if let Some(last_obj_end) = trimmed.rfind('}') {
        let mut sliced = trimmed[..=last_obj_end].trim().to_string();
        let open_brackets = sliced.chars().filter(|&c| c == '[').count();
        let close_brackets = sliced.chars().filter(|&c| c == ']').count();
        let open_braces = sliced.chars().filter(|&c| c == '{').count();
        let close_braces = sliced.chars().filter(|&c| c == '}').count();
        if open_brackets > close_brackets {
            for _ in 0..(open_brackets - close_brackets) {
                sliced.push(']');
            }
        }
        if open_braces > close_braces {
            for _ in 0..(open_braces - close_braces) {
                sliced.push('}');
            }
        }
        return sliced;
    }
    trimmed.to_string()
}

pub fn extract_json_block(raw: &str) -> String {
    let clean = raw.trim();
    let json_candidate = if let Some(start) = clean.find("```") {
        let after_fence = &clean[start + 3..];
        let content_start = if let Some(newline) = after_fence.find('\n') {
            &after_fence[newline + 1..]
        } else {
            after_fence
        };
        if let Some(end) = content_start.rfind("```") {
            &content_start[..end]
        } else {
            content_start
        }
    } else {
        clean
    };
    let sanitized = sanitize_lenient_json(json_candidate.trim());
    let repaired = repair_truncated_json(&sanitized);
    let trimmed = repaired.trim();
    match (trimmed.find('{'), trimmed.rfind('}'), trimmed.find('['), trimmed.rfind(']')) {
        (Some(fo), Some(lo), Some(fa), Some(la)) => {
            if fa < fo && la > lo {
                trimmed[fa..=la].to_string()
            } else {
                trimmed[fo..=lo].to_string()
            }
        }
        (Some(fo), Some(lo), _, _) => trimmed[fo..=lo].to_string(),
        (_, _, Some(fa), Some(la)) => trimmed[fa..=la].to_string(),
        _ => trimmed.to_string(),
    }
}

pub fn sanitize_course_title(raw: &str) -> String {
    let trimmed = raw.trim();
    if let Some(pos) = trimmed.find(" - ") {
        let prefix = &trimmed[..pos];
        if prefix.chars().all(|c| c.is_ascii_digit() || c.is_whitespace()) {
            return trimmed[pos + 3..].trim().to_string();
        }
    }
    trimmed.to_string()
}

pub fn clean_and_parse_lessons_json(text: &str) -> Result<Vec<ExtractedLessonItem>, String> {
    let clean = text.trim();
    if clean.is_empty() {
        return Err("Risposta AI vuota.".to_string());
    }
    let json_slice = extract_json_block(clean);
    let parsed: serde_json::Value = serde_json::from_str(&json_slice)
        .map_err(|e| format!("Impossibile interpretare l'orario JSON: {}", e))?;

    let items_array_opt = if let Some(arr) = parsed.as_array() {
        Some(arr.clone())
    } else if let Some(obj) = parsed.as_object() {
        let candidate_keys = ["lessons", "timetable", "schedule", "orario", "classes", "items", "lezioni", "data"];
        let mut found = None;
        for key in &candidate_keys {
            if let Some(val) = obj.get(*key) {
                if let Some(arr) = val.as_array() {
                    found = Some(arr.clone());
                    break;
                }
            }
        }
        if found.is_none() {
            for val in obj.values() {
                if let Some(arr) = val.as_array() {
                    found = Some(arr.clone());
                    break;
                }
            }
        }
        found
    } else {
        None
    };

    let mut raw_lessons = Vec::new();
    if let Some(items_array) = items_array_opt {
        let course_aliases = ["courseName", "course_name", "course", "materia", "subject", "title", "nome_corso", "lezione", "name"];
        let start_aliases = ["startTime", "start_time", "start", "ora_inizio", "inizio", "from", "oraInizio"];
        let end_aliases = ["endTime", "end_time", "end", "ora_fine", "fine", "to", "oraFine"];
        let combined_time_aliases = ["time", "orario", "hours", "fascia_oraria", "fasciaOraria", "slot"];
        let day_aliases = ["dayOfWeek", "day_of_week", "day", "giorno", "weekday"];
        let classroom_aliases = ["classroom", "aula", "room", "location"];
        let professor_aliases = ["professor", "docente", "teacher", "prof"];

        let mut last_course_name = String::new();
        let mut last_professor = String::new();

        for item in items_array {
            if let Some(obj) = item.as_object() {
                let mut day = 1;
                for alias in &day_aliases {
                    if let Some(val) = obj.get(*alias) {
                        if let Some(d) = parse_day_of_week_value(val) {
                            day = d;
                            break;
                        }
                    }
                }
                let mut start_raw = String::new();
                for alias in &start_aliases {
                    if let Some(val) = obj.get(*alias) {
                        if let Some(s) = val.as_str() {
                            start_raw = s.trim().to_string();
                            break;
                        } else if let Some(n) = val.as_i64() {
                            start_raw = n.to_string();
                            break;
                        }
                    }
                }
                let mut end_raw = String::new();
                for alias in &end_aliases {
                    if let Some(val) = obj.get(*alias) {
                        if let Some(s) = val.as_str() {
                            end_raw = s.trim().to_string();
                            break;
                        } else if let Some(n) = val.as_i64() {
                            end_raw = n.to_string();
                            break;
                        }
                    }
                }
                let mut norm_start = normalize_hour_string(&start_raw);
                let mut norm_end = normalize_hour_string(&end_raw);
                if norm_start.is_none() || norm_end.is_none() {
                    for alias in &combined_time_aliases {
                        if let Some(val) = obj.get(*alias).and_then(|v| v.as_str()) {
                            let parts: Vec<&str> = val.split(|c| c == '-' || c == '—' || c == '/' || c == '–').collect();
                            if parts.len() >= 2 {
                                if norm_start.is_none() {
                                    norm_start = normalize_hour_string(parts[0]);
                                }
                                if norm_end.is_none() {
                                    norm_end = normalize_hour_string(parts[1]);
                                }
                            }
                        }
                    }
                }

                let course_name = course_aliases
                    .iter()
                    .find_map(|a| obj.get(*a).and_then(|v| v.as_str()).map(|s| s.trim().to_string()))
                    .filter(|s| !s.is_empty())
                    .unwrap_or_default();
                let classroom = classroom_aliases
                    .iter()
                    .find_map(|a| obj.get(*a).and_then(|v| v.as_str()).map(|s| s.trim().to_string()))
                    .unwrap_or_default();
                let professor = professor_aliases
                    .iter()
                    .find_map(|a| obj.get(*a).and_then(|v| v.as_str()).map(|s| s.trim().to_string()))
                    .unwrap_or_default();

                let clean_title = sanitize_course_title(&course_name);
                let final_course = if !clean_title.is_empty() {
                    clean_title
                } else if !last_course_name.is_empty() {
                    last_course_name.clone()
                } else {
                    "Lezione".to_string()
                };
                let final_prof = if professor.is_empty() && final_course.eq_ignore_ascii_case(&last_course_name) {
                    last_professor.clone()
                } else {
                    professor
                };
                last_course_name = final_course.clone();
                last_professor = final_prof.clone();

                raw_lessons.push(ExtractedLessonItem {
                    day_of_week: day,
                    start_time: norm_start.unwrap_or_else(|| "09:00".to_string()),
                    end_time: norm_end.unwrap_or_else(|| "11:00".to_string()),
                    course_name: final_course,
                    classroom,
                    professor: final_prof,
                });
            }
        }
    }

    let mut unique_lessons = Vec::new();
    for item in raw_lessons {
        let is_dup = unique_lessons.iter().any(|ex: &ExtractedLessonItem| {
            ex.day_of_week == item.day_of_week
                && ex.start_time == item.start_time
                && ex.end_time == item.end_time
                && ex.course_name.eq_ignore_ascii_case(&item.course_name)
        });
        if !is_dup {
            unique_lessons.push(item);
        }
    }

    if unique_lessons.is_empty() {
        return Err("Nessuna lezione identificata nell'orario.".to_string());
    }
    Ok(unique_lessons)
}
