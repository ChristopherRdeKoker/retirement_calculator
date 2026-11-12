const { chromium } = await import(process.env.PLAYWRIGHT_MODULE ?? "playwright");

const URL = `http://127.0.0.1:${process.env.PORT ?? "8731"}/apps/retirement/`;
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
  const resp = await page.goto(URL);
  check("CSP header served", (resp.headers()["content-security-policy"] || "").includes("wasm-unsafe-eval"));
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
  await page.goto(URL);
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
  await page.goto(URL);
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
  await page.goto(URL);
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
  const ctx = await browser.newContext({ viewport: { width: 390, height: 844 } });
  const page = await ctx.newPage();
  const consoleErrors = [];
  page.on("console", (m) => { if (m.type() === "error") consoleErrors.push(m.text()); });
  await page.route(URL, async (route) => {
    const response = await route.fetch();
    const body = (await response.text()).replace('<div id="app">', '<div id="not-app">');
    await route.fulfill({ response, body });
  });
  await page.goto(URL);
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
