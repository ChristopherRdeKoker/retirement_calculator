const { chromium } = await import(process.env.PLAYWRIGHT_MODULE ?? "playwright");
const { mkdirSync } = await import("node:fs");

const APP_URL = `http://127.0.0.1:${process.env.PORT ?? "8731"}/apps/retirement/`;
const SHOTS = process.env.SHOTS_DIR ?? new URL("screenshots", import.meta.url).pathname;
const problems = [];
const checks = [];
function check(name, ok, detail = "") {
  checks.push(`${ok ? "PASS" : "FAIL"} ${name}${detail ? " :: " + detail : ""}`);
  if (!ok) problems.push(name);
}

function watch(page, label, errors) {
  page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") errors.push(`${label} console.${m.type()}: ${m.text()}`); });
  page.on("pageerror", (e) => errors.push(`${label} pageerror: ${e.message}`));
  page.on("requestfailed", (r) => errors.push(`${label} requestfailed: ${r.url()} ${r.failure()?.errorText}`));
  page.on("response", (r) => { if (r.status() >= 400) errors.push(`${label} http ${r.status()}: ${r.url()}`); });
}

const csp = `window.__csp = []; document.addEventListener("securitypolicyviolation", (e) => window.__csp.push(e.violatedDirective + " " + e.blockedURI));`;

const browser = await chromium.launch();
const errors = [];

async function field(page, id) { return page.locator(`#${id}`); }
async function setField(page, id, value) {
  const f = await field(page, id);
  await f.fill(value);
  await f.blur();
}
async function headline(page) { return (await page.locator("#summary-title").textContent()).trim(); }

{
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, colorScheme: "light", hasTouch: true, isMobile: true });
  const page = await ctx.newPage();
  await page.addInitScript(csp);
  watch(page, "phone", errors);
  const resp = await page.goto(APP_URL);
  check("CSP header served", (resp.headers()["content-security-policy"] || "").includes("wasm-unsafe-eval"));
  const metaCsp = (await resp.text()).match(/<meta http-equiv="Content-Security-Policy" content="([^"]*)"/);
  check("page carries its own CSP meta (default-src 'none', wasm only via script-src)", !!metaCsp && metaCsp[1].includes("default-src 'none'") && metaCsp[1].includes("script-src 'self' 'wasm-unsafe-eval'") && !metaCsp[1].includes("'unsafe-eval'") && !metaCsp[1].includes("'unsafe-inline'"), metaCsp ? metaCsp[1] : "no meta");
  await page.waitForSelector("#summary-title");
  const h0 = await headline(page);
  check("default headline", /^Your savings last (until|beyond) age \d+$/.test(h0), h0);
  check("rand formatting", /R \d{1,3}( \d{3})+/.test(await page.locator(".stat-value").first().textContent()));
  check("chart svg rendered", (await page.locator("svg.chart path.line").count()) === 1);

  await setField(page, "retire-age", "30");
  const bad = await headline(page);
  check("validation shown for retire <= current", bad === "Check your inputs" && (await page.locator("#retire-age-error").textContent()).includes("later than your current age"), bad);
  check("aria-invalid set", (await page.locator("#retire-age").getAttribute("aria-invalid")) === "true");
  check("save disabled while invalid", await page.locator(".save-row button").isDisabled());

  await setField(page, "retire-age", "60");
  await setField(page, "contribution", "8000");
  const h1 = await headline(page);
  check("headline recomputed", /^Your savings last/.test(h1), `${h0} -> ${h1}`);
  check("money reformatted on blur", (await (await field(page, "contribution")).inputValue()) === "8 000");

  await page.locator("#scenario-name").fill("Retire at 60");
  await page.locator(".save-row button").click();
  await setField(page, "retire-age", "67");
  await page.locator("#scenario-name").fill("Work to 67");
  await page.locator(".save-row button").click();
  check("two scenarios listed", (await page.locator(".scenario").count()) === 2);
  check("compare table has 2 columns", (await page.locator(".compare thead th").count()) === 2);

  await page.locator("svg.chart rect.hit").tap();
  await page.waitForTimeout(150);
  check("chart tooltip on tap", (await page.locator("svg.chart .tip-value").count()) === 1);

  await page.reload();
  await page.waitForSelector("#summary-title");
  check("inputs persisted after reload", (await (await field(page, "retire-age")).inputValue()) === "67" && (await (await field(page, "contribution")).inputValue()) === "8 000");
  check("scenarios persisted after reload", (await page.locator(".scenario-name").allTextContents()).join("|") === "Retire at 60|Work to 67");

  await page.getByRole("button", { name: "Load Retire at 60" }).click();
  check("load scenario restores inputs", (await (await field(page, "retire-age")).inputValue()) === "60");

  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check("no horizontal scroll at 390", overflow <= 0, `overflow=${overflow}`);
  await page.screenshot({ path: `${SHOTS}/phone-390x844.png`, fullPage: false });
  await page.screenshot({ path: `${SHOTS}/phone-390x844-full.png`, fullPage: true });

  await page.locator("#scenario-name").fill("Third");
  await page.locator(".save-row button").click();
  await page.locator("#scenario-name").fill("Fourth");
  await page.locator(".save-row button").click();
  check("limit of 3 enforced", (await page.locator(".scenario").count()) === 3 && (await page.locator(".notice").textContent()).includes("Delete one"));
  await page.getByRole("button", { name: "Delete Third" }).click();
  check("delete works", (await page.locator(".scenario").count()) === 2);

  const v = await page.evaluate(() => window.__csp);
  check("no CSP violations (phone)", v.length === 0, JSON.stringify(v));

  await page.evaluate(() => document.body.setAttribute("style", "color:red"));
  await page.waitForTimeout(100);
  const ctrl = await page.evaluate(() => window.__csp);
  check("CSP control: inline style IS blocked (CSP is live)", ctrl.some((x) => x.startsWith("style-src")), JSON.stringify(ctrl));
  const before = errors.length;
  await page.waitForTimeout(100);
  errors.splice(before);
  for (let i = errors.length - 1; i >= 0; i--) if (errors[i].includes("Content Security Policy") && errors[i].includes("style")) errors.splice(i, 1);
  await ctx.close();
}

{
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, colorScheme: "dark" });
  const page = await ctx.newPage();
  await page.addInitScript(csp);
  watch(page, "corrupt", errors);
  await page.goto(APP_URL);
  await page.evaluate(() => {
    localStorage.setItem("retire-calc.inputs.v1", "{not json");
    localStorage.setItem("retire-calc.scenarios.v1", JSON.stringify([{ name: "ok", inputs: { current_age: 500 } }, 7, null, "x"]));
  });
  await page.reload();
  await page.waitForSelector("#summary-title");
  check("corrupt storage -> defaults", (await (await field(page, "current-age")).inputValue()) === "35" && (await page.locator(".scenario").count()) === 0);
  check("corrupt storage -> notice", (await page.locator(".notice").textContent()).includes("could not be read"));
  await page.evaluate(() => {
    localStorage.setItem("retire-calc.inputs.v1", JSON.stringify({ current_age: 70, retire_age: 60, life_expectancy: 90, savings: -5, monthly_contribution: 1, contribution_increase: 1, return_pre: 1, return_post: 1, inflation: 1, desired_income: 1 }));
    localStorage.setItem("retire-calc.scenarios.v1", "\"a string\"");
  });
  await page.reload();
  await page.waitForSelector("#summary-title");
  check("parseable-but-invalid storage -> defaults", (await (await field(page, "current-age")).inputValue()) === "35");
  await page.evaluate(() => localStorage.clear());
  await page.reload();
  await page.waitForSelector("#summary-title");
  await setField(page, "income", "60000");
  check("unsustainable verdict shown", (await page.locator(".verdict-bad").count()) === 1);
  await page.evaluate(() => window.scrollTo(0, 0));
  await page.screenshot({ path: `${SHOTS}/phone-390x844-dark.png` });
  await page.screenshot({ path: `${SHOTS}/phone-390x844-dark-full.png`, fullPage: true });
  const v = await page.evaluate(() => window.__csp);
  check("no CSP violations (dark)", v.length === 0, JSON.stringify(v));
  await ctx.close();
}

{
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: "light" });
  const page = await ctx.newPage();
  await page.addInitScript(csp);
  watch(page, "desktop", errors);
  await page.goto(APP_URL);
  await page.waitForSelector("#summary-title");
  for (const [n, r] of [["Retire at 60", "60"], ["Baseline", "65"], ["Work to 70", "70"]]) {
    await setField(page, "retire-age", r);
    await page.locator("#scenario-name").fill(n);
    await page.locator(".save-row button").click();
  }
  await setField(page, "retire-age", "65");
  const box = await page.locator("svg.chart rect.hit").boundingBox();
  await page.mouse.move(box.x + box.width * 0.5, box.y + box.height * 0.5);
  check("chart tooltip on mouse hover", (await page.locator("svg.chart .tip-value").count()) === 1);
  await page.mouse.move(5, 5);
  check("tooltip hides on mouse leave", (await page.locator("svg.chart .tip-value").count()) === 0);
  await page.locator("svg.chart").focus();
  await page.keyboard.press("End");
  for (let i = 0; i < 12; i++) await page.keyboard.press("ArrowLeft");
  check("keyboard moves chart tooltip", (await page.locator("svg.chart .tip-title").textContent()) === "Age 83");
  await page.evaluate(() => window.scrollTo(0, 0));
  await page.screenshot({ path: `${SHOTS}/desktop-1440x900.png` });
  await page.screenshot({ path: `${SHOTS}/desktop-1440x900-full.png`, fullPage: true });
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check("no horizontal scroll at 1440", overflow <= 0, `overflow=${overflow}`);
  const v = await page.evaluate(() => window.__csp);
  check("no CSP violations (desktop)", v.length === 0, JSON.stringify(v));
  await ctx.close();
}

{
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, colorScheme: "light", hasTouch: true, isMobile: true });
  const page = await ctx.newPage();
  await page.addInitScript(csp);
  watch(page, "phone3", errors);
  await page.goto(APP_URL);
  await page.waitForSelector("svg.chart");
  await page.evaluate(() => {
    window.__widths = [];
    window.__wrap = document.querySelector(".chart-wrap");
    new MutationObserver((ms) => { for (const m of ms) if (m.target.matches?.("svg.chart") || m.target.querySelector?.("svg.chart")) { const svg = document.querySelector("svg.chart"); if (svg) window.__widths.push(svg.getAttribute("width")); } })
      .observe(document.querySelector(".area-chart"), { subtree: true, childList: true, attributes: true, attributeFilter: ["width"] });
  });
  const w0 = await page.evaluate(() => document.querySelector("svg.chart").getAttribute("width"));
  await page.locator("#contribution").pressSequentially("1");
  await page.locator("#return-pre").fill("11");
  await page.waitForTimeout(100);
  const r = await page.evaluate(() => ({ same: window.__wrap === document.querySelector(".chart-wrap"), widths: window.__widths }));
  check("chart not remounted on input", r.same, JSON.stringify(r));
  check("chart never painted at fallback 640 on phone", !r.widths.some((w) => w.startsWith("640")) && w0 !== "640.0", `${w0} ${JSON.stringify(r.widths)}`);
  for (const [n, ret] of [["Early retirement at 55 with lump sum", "55"], ["Baseline plan", "65"], ["Keep working until 70 years", "70"]]) {
    await setField(page, "retire-age", ret);
    await page.locator("#scenario-name").fill(n);
    await page.locator(".save-row button").click();
  }
  check("three scenarios saved", (await page.locator(".scenario").count()) === 3);
  const t = await page.evaluate(() => { const el = document.querySelector(".compare").parentElement; return [el.scrollWidth, el.clientWidth]; });
  check("compare table fits at 390 with 3 scenarios", t[0] <= t[1], JSON.stringify(t));
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check("no horizontal scroll at 390 with 3 scenarios", overflow <= 0, `overflow=${overflow}`);
  await page.locator(".area-scenarios").screenshot({ path: `${SHOTS}/phone-390-three-scenarios.png` });
  const v = await page.evaluate(() => window.__csp);
  check("no CSP violations (phone3)", v.length === 0, JSON.stringify(v));
  await ctx.close();
}

{
  const HELP_SHOTS = `${SHOTS}/help`;
  mkdirSync(HELP_SHOTS, { recursive: true });
  const states = (page) => page.locator(".help-toggle").evaluateAll((els) => els.map((el) => el.getAttribute("aria-expanded")));
  const openCount = (page) => page.locator(".help:not([hidden])").count();

  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, colorScheme: "light", hasTouch: true, isMobile: true });
  const page = await ctx.newPage();
  await page.addInitScript(csp);
  watch(page, "help-phone", errors);
  await page.goto(APP_URL);
  await page.waitForSelector("#summary-title");
  const h0 = await headline(page);
  const toggles = page.locator(".group .help-toggle");
  check("help toggle on every field", (await toggles.count()) === 10 && (await page.locator(".group .field").count()) === 10);
  const wiring = await toggles.evaluateAll((els) => els.map((el) => {
    const help = document.getElementById(el.getAttribute("aria-controls") || "");
    const input = el.closest(".field").querySelector("input");
    return { type: el.type, name: el.getAttribute("aria-label"), expanded: el.getAttribute("aria-expanded"), help: !!help, hidden: help?.hidden, text: (help?.textContent || "").trim().length, label: el.closest(".field").querySelector("label").textContent, input: input.id };
  }));
  check("help toggles wired: type=button, named, collapsed, controls a hidden non-empty panel", wiring.every((w) => w.type === "button" && w.name === `About ${w.label}` && w.expanded === "false" && w.help && w.hidden === true && w.text > 20), JSON.stringify(wiring.filter((w) => !(w.type === "button" && w.expanded === "false" && w.help && w.hidden))));

  const rp = page.getByRole("button", { name: "About Return before retirement" });
  await rp.tap();
  check("tap opens help", (await rp.getAttribute("aria-expanded")) === "true" && (await page.locator("#return-pre-help").isVisible()) && (await page.locator("#return-pre-help").textContent()).includes("before inflation"));
  check("open help described on input", ((await page.locator("#return-pre").getAttribute("aria-describedby")) || "").split(" ").includes("return-pre-help"));
  check("closed help not described on input", !((await page.locator("#savings").getAttribute("aria-describedby")) || "").includes("savings-help"));
  const helpBox = await page.locator("#return-pre-help").boundingBox();
  check("open help fits the 390 screen", helpBox.x >= 0 && helpBox.x + helpBox.width <= 390, JSON.stringify(helpBox));
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check("no horizontal scroll at 390 with help open", overflow <= 0, `overflow=${overflow}`);
  await page.locator("#return-pre-help").scrollIntoViewIfNeeded();
  await page.evaluate(() => window.scrollBy(0, 120));
  await page.screenshot({ path: `${HELP_SHOTS}/phone-390x844-light.png` });

  const inf = page.getByRole("button", { name: "About Inflation" });
  await inf.tap();
  check("tapping another help closes the open one", (await openCount(page)) === 1 && (await rp.getAttribute("aria-expanded")) === "false" && (await inf.getAttribute("aria-expanded")) === "true" && (await page.locator("#return-pre-help").isHidden()));
  await inf.tap();
  check("tapping the same help closes it", (await openCount(page)) === 0 && (await states(page)).every((x) => x === "false"));
  check("toggling help does not submit or change results", (await headline(page)) === h0 && new URL(page.url()).search === "");
  const v = await page.evaluate(() => window.__csp);
  check("no CSP violations (help phone)", v.length === 0, JSON.stringify(v));
  await ctx.close();
}

for (const scheme of ["light", "dark"]) {
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: scheme });
  const page = await ctx.newPage();
  await page.addInitScript(csp);
  watch(page, `help-desktop-${scheme}`, errors);
  await page.goto(APP_URL);
  await page.waitForSelector("#summary-title");
  const openCount = () => page.locator(".help:not([hidden])").count();
  const rp = page.getByRole("button", { name: "About Return before retirement" });
  if (scheme === "light") {
    await rp.click();
    check("click opens help", (await rp.getAttribute("aria-expanded")) === "true" && (await openCount()) === 1);
    await rp.click();
    check("click again closes help", (await rp.getAttribute("aria-expanded")) === "false" && (await openCount()) === 0);

    await page.locator("#return-pre").focus();
    await page.keyboard.press("Shift+Tab");
    check("help toggle reachable by keyboard", await rp.evaluate((el) => el === document.activeElement));
    await page.keyboard.press("Enter");
    check("Enter opens help", (await rp.getAttribute("aria-expanded")) === "true" && (await openCount()) === 1);
    await page.keyboard.press("Space");
    check("Space closes help", (await rp.getAttribute("aria-expanded")) === "false" && (await openCount()) === 0);
    await page.keyboard.press("Space");
    await page.keyboard.press("Escape");
    check("Escape closes help and keeps focus on the toggle", (await rp.getAttribute("aria-expanded")) === "false" && (await openCount()) === 0 && (await rp.evaluate((el) => el === document.activeElement)));
    await page.keyboard.press("Enter");
    await page.keyboard.press("Tab");
    check("Tab moves from toggle to its input", await page.locator("#return-pre").evaluate((el) => el === document.activeElement));
    const openBeforeEscape = await openCount();
    await page.keyboard.press("Escape");
    check("Escape from the input closes help, focus stays in input", openBeforeEscape === 1 && (await openCount()) === 0 && (await page.locator("#return-pre").evaluate((el) => el === document.activeElement)), `open before=${openBeforeEscape}`);
    await page.getByRole("button", { name: "About Current savings" }).click();
    await rp.click();
    check("only one help open at a time (desktop)", (await openCount()) === 1 && (await page.locator("#savings-help").isHidden()));
  } else {
    await rp.click();
  }
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check(`no horizontal scroll at 1440 with help open (${scheme})`, overflow <= 0, `overflow=${overflow}`);
  await page.locator("#return-pre-help").evaluate((el) => el.scrollIntoView({ block: "center" }));
  await page.mouse.move(5, 5);
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${SHOTS}/help/desktop-1440x900-${scheme}.png` });
  const v = await page.evaluate(() => window.__csp);
  check(`no CSP violations (help desktop ${scheme})`, v.length === 0, JSON.stringify(v));
  await ctx.close();
}

{
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, colorScheme: "dark", hasTouch: true, isMobile: true });
  const page = await ctx.newPage();
  await page.addInitScript(csp);
  watch(page, "help-phone-dark", errors);
  await page.goto(APP_URL);
  await page.waitForSelector("#summary-title");
  await page.getByRole("button", { name: "About Return before retirement" }).tap();
  await page.locator("#return-pre-help").scrollIntoViewIfNeeded();
  await page.evaluate(() => window.scrollBy(0, 120));
  await page.screenshot({ path: `${SHOTS}/help/phone-390x844-dark.png` });
  const v = await page.evaluate(() => window.__csp);
  check("no CSP violations (help phone dark)", v.length === 0, JSON.stringify(v));
  await ctx.close();
}

{
  const DEPOSIT_SHOTS = `${SHOTS}/deposits`;
  mkdirSync(DEPOSIT_SHOTS, { recursive: true });
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 }, deviceScaleFactor: 2, colorScheme: "light", hasTouch: true, isMobile: true });
  const page = await ctx.newPage();
  await page.addInitScript(csp);
  watch(page, "deposits-phone", errors);
  await page.goto(APP_URL);
  await page.waitForSelector("#summary-title");
  const pot = () => page.locator(".stat-value").first().textContent();
  const pot0 = await pot();
  const toggle = page.getByRole("button", { name: "Deposit at start of year", exact: true });
  check("deposit section titled and closed by default", (await toggle.textContent()).trim() === "Deposit at start of year" && (await toggle.getAttribute("aria-expanded")) === "false" && (await page.locator("#deposits-body").isHidden()));
  const sectionHelp = page.getByRole("button", { name: "About Deposit at start of year" });
  await sectionHelp.tap();
  check("deposit section help opens", (await sectionHelp.getAttribute("aria-expanded")) === "true" && (await page.locator("#deposits-help").isVisible()) && (await page.locator("#deposits-help").textContent()).includes("start of the year"));
  await sectionHelp.tap();
  await toggle.tap();
  check("deposit section opens with no rows", (await toggle.getAttribute("aria-expanded")) === "true" && (await page.locator(".deposit-row").count()) === 0 && (await page.getByRole("button", { name: "Add deposit" }).isVisible()));
  await page.getByRole("button", { name: "Add deposit" }).tap();
  const year = page.locator(".deposit-row input[name$='-year']").first();
  const amount = page.locator(".deposit-row input[name$='-amount']").first();
  check("new deposit row starts in year 1 with no amount, blocking the result", (await page.locator(".deposit-row").count()) === 1 && (await year.inputValue()) === "1" && (await headline(page)) === "Check your inputs");
  check("new deposit row shows no error before the amount is touched", (await amount.getAttribute("aria-invalid")) === "false" && (await page.locator(".deposit-row .error").count()) === 0);
  await amount.focus();
  await amount.blur();
  check("leaving the deposit amount empty shows its error", (await amount.getAttribute("aria-invalid")) === "true" && (await page.locator(".deposit-row .error").textContent()).includes("Deposit 1: enter the amount"));
  await amount.tap();
  await amount.pressSequentially("100000");
  check("typing in a deposit keeps focus", await amount.evaluate((el) => el === document.activeElement) && (await amount.inputValue()) === "100000");
  await amount.blur();
  await year.fill("3");
  await year.blur();
  const pot1 = await pot();
  check("deposit changes the result", /^Your savings last/.test(await headline(page)) && pot1 !== pot0, `${pot0} -> ${pot1}`);
  check("deposit amount regrouped on blur", (await amount.inputValue()) === "100 000");
  const yearHelp = page.getByRole("button", { name: "About deposit year" });
  await yearHelp.tap();
  const yearHelpId = await yearHelp.getAttribute("aria-controls");
  check("deposit year help opens", (await yearHelp.getAttribute("aria-expanded")) === "true" && (await page.locator(`#${yearHelpId}`).isVisible()) && (await page.locator(".help:not([hidden])").count()) === 1);
  const amountHelp = page.getByRole("button", { name: "About deposit amount" });
  await amountHelp.tap();
  check("deposit amount help opens and closes the year help", (await amountHelp.getAttribute("aria-expanded")) === "true" && (await yearHelp.getAttribute("aria-expanded")) === "false");
  await amountHelp.tap();
  for (const bad of ["0", "31"]) {
    await year.fill(bad);
    await year.blur();
    const errorId = `${await year.getAttribute("id")}-error`;
    check(`deposit year ${bad} is rejected`, (await headline(page)) === "Check your inputs" && (await page.locator(`#${errorId}`).textContent()).includes("Deposit 1: the year"));
  }
  await year.fill("30");
  await year.blur();
  check("deposit in the last year before retirement is allowed", /^Your savings last/.test(await headline(page)));
  await year.fill("3");
  await year.blur();
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check("no horizontal scroll at 390 with a deposit row", overflow <= 0, `overflow=${overflow}`);
  await page.locator(".deposits").scrollIntoViewIfNeeded();
  await page.locator(".deposits").screenshot({ path: `${DEPOSIT_SHOTS}/phone-390-section.png` });
  await page.screenshot({ path: `${DEPOSIT_SHOTS}/phone-390x844.png` });

  await page.locator("#scenario-name").fill("With deposit");
  await page.locator(".save-row button").click();
  check("compare table shows deposits", (await page.locator(".compare tbody tr", { hasText: "Deposits" }).locator("td").first().textContent()).trim() !== "None");
  await page.reload();
  await page.waitForSelector("#summary-title");
  check("deposit kept after reload and section opens", (await toggle.getAttribute("aria-expanded")) === "true" && (await page.locator(".deposit-row").count()) === 1 && (await pot()) === pot1);
  await page.getByRole("button", { name: "Reset" }).tap();
  check("reset clears deposits and closes the section", (await page.locator(".deposit-row").count()) === 0 && (await toggle.getAttribute("aria-expanded")) === "false" && (await pot()) === pot0);
  await page.getByRole("button", { name: "Load With deposit" }).tap();
  check("loading a scenario restores its deposit", (await page.locator(".deposit-row").count()) === 1 && (await pot()) === pot1);
  await page.getByRole("button", { name: "Remove deposit 1" }).tap();
  check("remove deposit restores the result", (await page.locator(".deposit-row").count()) === 0 && (await pot()) === pot0);
  const v = await page.evaluate(() => window.__csp);
  check("no CSP violations (deposits phone)", v.length === 0, JSON.stringify(v));
  await ctx.close();
}

{
  const ctx = await browser.newContext({ viewport: { width: 1440, height: 900 }, colorScheme: "light" });
  const page = await ctx.newPage();
  await page.addInitScript(csp);
  watch(page, "deposits-desktop", errors);
  await page.goto(APP_URL);
  await page.waitForSelector("#summary-title");
  await page.getByRole("button", { name: "Deposit at start of year", exact: true }).click();
  await page.getByRole("button", { name: "Add deposit" }).click();
  await page.locator(".deposit-row input[name$='-year']").first().fill("3");
  await page.locator(".deposit-row input[name$='-amount']").first().fill("100000");
  await page.locator(".deposit-row input[name$='-amount']").first().blur();
  await page.getByRole("button", { name: "About deposit year" }).click();
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth - window.innerWidth);
  check("no horizontal scroll at 1440 with a deposit row", overflow <= 0, `overflow=${overflow}`);
  await page.locator(".deposits").evaluate((el) => el.scrollIntoView({ block: "center" }));
  await page.mouse.move(5, 5);
  await page.waitForTimeout(300);
  await page.screenshot({ path: `${SHOTS}/deposits/desktop-1440x900.png` });
  const v = await page.evaluate(() => window.__csp);
  check("no CSP violations (deposits desktop)", v.length === 0, JSON.stringify(v));
  await ctx.close();
}

{
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await ctx.newPage();
  const consoleErrors = [];
  page.on("console", (m) => { if (m.type() === "error") consoleErrors.push(m.text()); });
  await page.route(APP_URL, async (route) => {
    const response = await route.fetch();
    const body = (await response.text()).replace('<div id="app">', '<div id="not-app">');
    await route.fulfill({ response, body });
  });
  await page.goto(APP_URL);
  await page.waitForTimeout(1500);
  check("missing #app logs a console error", consoleErrors.some((e) => e.includes("no #app element")), JSON.stringify(consoleErrors));
  await ctx.close();
}

await browser.close();
console.log(checks.join("\n"));
console.log(`console/page/network errors: ${errors.length}`);
for (const e of errors) console.log("  " + e);
console.log(problems.length || errors.length ? `RESULT: FAIL (${problems.length} checks, ${errors.length} errors)` : `RESULT: PASS (${checks.length} checks, 0 errors)`);
process.exit(problems.length || errors.length ? 1 : 0);
