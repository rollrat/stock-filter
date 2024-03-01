from pathlib import Path
import pandas as pd
import progressbar
import typer
import FinanceDataReader as fdr

app = typer.Typer()

market_list = [
    "KRX",
    "KOSPI",
    "KOSDAQ",
    "KONEX",
    "KRX-MARCAP",
    "KRX-DESC",
    "KOSPI-DESC",
    "KOSDAQ-DESC",
    "KONEX-DESC",
    "NASDAQ",
    "NYSE",
    "AMEX",
    "SSE",
    "SZSE",
    "HKEX",
    "TSE",
    "HOSE",
    "KRX-DELISTING",
    "KRX-ADMINISTRATIVE",
    "S&P500",
    "SP500",
]


@app.command()
def list_tickers(market: str):
    if market not in market_list:
        print(f"{market} market is not supports")
        exit(0)

    df = fdr.StockListing(market)
    code = df["Code"] if "Code" in df.columns else df["Symbol"]
    name = df["Name"]
    df = pd.DataFrame({"code": code, "name": name})

    path = Path(f"data")
    path.mkdir(parents=True, exist_ok=True)
    df.to_csv(f"data/{market}.txt")


@app.command()
def prices(code: str):
    df = fdr.DataReader(code, "2018")
    print(df.tail())


@app.command()
def market_prices(market: str):
    if market not in market_list:
        print(f"{market} market is not supports")
        exit(0)

    df = fdr.StockListing(market)
    codes = df["Code"] if "Code" in df.columns else df["Symbol"]

    bar = progressbar.ProgressBar(maxval=len(codes)).start()
    for i, code in enumerate(codes, start=0):
        df = fdr.DataReader(code, "2018")
        path = Path(f"data/{market}")
        path.mkdir(parents=True, exist_ok=True)
        df.to_csv(f"data/{market}/{code}")
        bar.update(i)


if __name__ == "__main__":
    app()
