/**
 * A flag beside each language on the switch: the name says the language, the
 * flag finds it at a glance. From flag-icons (MIT, github.com/lipis/flag-icons),
 * 4:3; Spain's is its civil flag, drawn here (the one with the arms is 80 KB).
 * English is shown with the United Kingdom's, Portuguese with Brazil's (the
 * translation is Brazilian), Arabic with Saudi Arabia's. Pictures only:
 * `alt=""` wherever one is shown, since the name beside it is what is read.
 */
import gb from "flag-icons/flags/4x3/gb.svg";
import ru from "flag-icons/flags/4x3/ru.svg";
import fr from "flag-icons/flags/4x3/fr.svg";
import de from "flag-icons/flags/4x3/de.svg";
import cn from "flag-icons/flags/4x3/cn.svg";
import jp from "flag-icons/flags/4x3/jp.svg";
import india from "flag-icons/flags/4x3/in.svg";
import br from "flag-icons/flags/4x3/br.svg";
import kr from "flag-icons/flags/4x3/kr.svg";
import sa from "flag-icons/flags/4x3/sa.svg";
import es from "../assets/flags/es.svg";

const FLAGS: Record<string, string> = { en: gb, ru, es, fr, de, zh: cn, ja: jp, hi: india, pt: br, ko: kr, ar: sa };

/** The flag of a language of the interface (its code), as an image URL. */
export const flagOf = (code: string): string => FLAGS[code] ?? "";
