---
name: package
description: "statsmodels package guide for Python statistical modeling, regression, hypothesis testing, and time series analysis"
metadata:
  languages: "python"
  versions: "0.14.6"
  revision: 1
  updated-on: "2026-03-12"
  source: maintainer
  tags: "statsmodels,python,statistics,regression,time-series,econometrics,glm,arima"
---

# statsmodels Python Package Guide

## What It Is

`statsmodels` is the main Python package for statistical modeling and inference: linear models, generalized linear models, discrete-choice models, mixed models, robust statistics, hypothesis tests, and classical time-series analysis.

Agent reminders:

- PyPI package name: `statsmodels`
- Import namespace: `statsmodels`
- Common convenience imports: `import statsmodels.api as sm` and `import statsmodels.formula.api as smf`
- This is a local numerical/statistical library. There is no API key, service auth, or remote client setup.

## Install

Install the exact version covered here:

```bash
python -m pip install "statsmodels==0.14.6"
```

For a fresh scientific-Python environment, it is reasonable to install the common core stack together:

```bash
python -m pip install "numpy" "scipy" "pandas" "patsy" "statsmodels==0.14.6"
```

With Conda:

```bash
conda install statsmodels
```

Verify the runtime package version before writing code against version-sensitive APIs:

```bash
python - <<'PY'
import statsmodels
print(statsmodels.__version__)
PY
```

## Initialize And Choose An API Style

Most projects use one of these import patterns:

```python
import statsmodels.api as sm
import statsmodels.formula.api as smf
```

Use the **array/matrix API** when your code already has prepared `numpy` arrays or `pandas` columns:

```python
import statsmodels.api as sm
```

Use the **formula API** when your data is in a `pandas.DataFrame` and you want Patsy formulas such as `y ~ x1 + x2 + C(group)`:

```python
import statsmodels.formula.api as smf
```

For time series, import directly from the relevant module or the time-series API namespace:

```python
from statsmodels.tsa.arima.model import ARIMA
```

## Core Usage

### Ordinary Least Squares With Explicit Design Matrix

For `sm.OLS`, add the intercept yourself unless your design matrix already contains one:

```python
import pandas as pd
import statsmodels.api as sm

df = pd.DataFrame(
    {
        "y": [2.2, 2.8, 3.6, 4.5, 5.1],
        "x1": [1.0, 2.0, 3.0, 4.0, 5.0],
        "x2": [5.0, 4.0, 3.0, 2.0, 1.0],
    }
)

X = sm.add_constant(df[["x1", "x2"]])
y = df["y"]

model = sm.OLS(y, X, missing="raise")
results = model.fit()

print(results.params)
print(results.rsquared)
print(results.summary())
```

Use `missing="raise"` or clean the data before fitting. The docs explicitly warn that silent missing-data handling can produce invalid all-`NaN` parameters.

### OLS With The Formula API

The formula API is usually the easiest path when your input is already a DataFrame:

```python
import pandas as pd
import statsmodels.formula.api as smf

df = pd.DataFrame(
    {
        "y": [2.2, 2.8, 3.6, 4.5, 5.1],
        "x1": [1.0, 2.0, 3.0, 4.0, 5.0],
        "group": ["a", "a", "b", "b", "b"],
    }
)

results = smf.ols("y ~ x1 + C(group)", data=df, missing="raise").fit()

print(results.params)
print(results.conf_int())
```

Formula models include an intercept by default. Use `0 +` or `- 1` in the formula only when you intentionally want a no-intercept model.

### Predictions And Robust Standard Errors

```python
robust_results = smf.ols("y ~ x1", data=df, missing="raise").fit(cov_type="HC3")

prediction = robust_results.get_prediction(df[["x1"]]).summary_frame()
print(prediction.head())
```

When you care about inference, check the covariance estimator you need instead of relying on defaults. `HC3` is a common heteroskedasticity-robust choice for linear models.

### Time-Series Forecasting With ARIMA

```python
import pandas as pd
from statsmodels.tsa.arima.model import ARIMA

series = pd.Series(
    [120, 128, 133, 142, 150, 160],
    index=pd.date_range("2025-01-01", periods=6, freq="MS"),
)

model = ARIMA(series, order=(1, 1, 1))
results = model.fit()

forecast = results.forecast(steps=3)
print(forecast)
```

For date-based forecasting, give the series a proper datetime-like index with a frequency. Otherwise forecast output can be harder to align and interpret.

## Common Modeling Surfaces

Use the API reference to jump to the right family:

- `sm.OLS`, `sm.WLS`, `sm.GLS`: classical linear regression
- `sm.GLM`: generalized linear models
- `sm.Logit`, `sm.Probit`, `sm.MNLogit`: discrete-choice models
- `sm.MixedLM`: mixed-effects models
- `sm.tsa`: ARIMA, state-space, exponential smoothing, seasonal decomposition, and related time-series tools
- `sm.stats`: tests, confidence intervals, and diagnostics

If you are unsure where a model lives, start from `statsmodels.api` or the official API reference instead of guessing import paths from blog posts.

## Configuration And Environment Notes

## Authentication

None. `statsmodels` runs locally.

## Environment And Dependencies

Practical setup issues are usually about the scientific Python stack, not package-level configuration:

- keep `numpy`, `scipy`, `pandas`, and `statsmodels` compatible inside one virtual environment
- install `patsy` if you use the formula API
- use `matplotlib` separately if your workflow needs plotting from examples or diagnostics
- prefer wheels over source builds unless you intentionally need a local compiled build

If import errors mention binary compatibility, check the versions of `numpy`, `scipy`, and Python before debugging application code.

## Common Pitfalls

### Missing data defaults

The official missing-data guide shows that default handling can let `NaN` values propagate into unusable parameter estimates. Use `missing="raise"` during development or clean/drop missing rows explicitly before fitting.

### Forgetting the intercept in matrix-based models

`sm.OLS(y, X)` does not add a constant automatically. Use `sm.add_constant(X)` unless the design matrix already includes one.

### Reusing a model instance across multiple fits

The pitfalls guide warns that fitting the same model instance repeatedly with different fit arguments can invalidate result objects because some model attributes are shared. Create separate model instances when you need multiple fits for comparison.

### Assuming rank-deficient or highly collinear data will raise

Linear-model code may silently proceed with generalized inverses. Check diagnostics such as condition numbers, singular design matrices, and warnings when coefficients look unstable.

### Treating convergence warnings as ignorable

Likelihood-based models can stop without clean convergence. Check `results.mle_retvals`, warnings, and parameter plausibility before trusting output from `Logit`, `GLM`, state-space, or other iterative estimators.

### Mixing stable and dev docs

`statsmodels` publishes both `stable` and `dev` docs. For `0.14.6`, prefer `https://www.statsmodels.org/stable/`. The `dev` docs reflect the in-progress `0.15.x` line and are not safe as a drop-in reference for `0.14.6`.

## Version-Sensitive Notes

- `0.14.6` is a maintenance release in the `0.14.x` line. The official release note highlights compatibility fixes for NumPy `2.4+` and pandas `3+`.
- PyPI currently lists Python `>=3.9` for `0.14.6`.
- The install page still includes older prose saying current support is Python `3.8`, `3.9`, and `3.10`. Treat that as stale documentation text rather than the release requirement for `0.14.6`.
- If you are diagnosing example differences, confirm whether the source you copied used the formula API or the matrix API. Intercept handling and categorical encoding differ between those entry points.

## Official Sources

- statsmodels package on PyPI: https://pypi.org/project/statsmodels/
- statsmodels stable docs root: https://www.statsmodels.org/stable/
- statsmodels install guide: https://www.statsmodels.org/stable/install.html
- statsmodels getting started guide: https://www.statsmodels.org/stable/gettingstarted.html
- statsmodels API reference: https://www.statsmodels.org/stable/api.html
- statsmodels import paths and structure: https://www.statsmodels.org/stable/api-structure.html
- statsmodels missing-data guide: https://www.statsmodels.org/stable/missing.html
- statsmodels pitfalls guide: https://www.statsmodels.org/stable/pitfalls.html
- statsmodels 0.14.6 release notes: https://www.statsmodels.org/stable/release/version0.14.6.html
