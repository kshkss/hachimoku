package main

import (
	//"context"
	"embed"
	"net/http"
	"strconv"
	//"strings"
	"time"

	"github.com/labstack/echo/v4"
	"github.com/labstack/echo/v4/middleware"

	"github.com/a-h/templ"
	// templ generate で生成されたパッケージをインポート
	// (モジュール名に合わせて変更してください 例: "github.com/yourname/hachimoku/views")
	"github.com/kshkss/hachimoku/views"
)

// render は Echo で templ コンポーネントをレンダリングするためのヘルパー関数です
func render(c echo.Context, status int, cmp templ.Component) error {
	c.Response().Writer.WriteHeader(status)
	c.Response().Header().Set(echo.HeaderContentType, echo.MIMETextHTML)
	return cmp.Render(c.Request().Context(), c.Response().Writer)
}

//go:embed assets/*
var assetsFS embed.FS

func main() {
	e := echo.New()

	e.Use(middleware.RequestLogger())
	e.Use(middleware.Recover())

	// 1. embed.FS から "assets" ディレクトリ以下を切り出す
	fsys := echo.MustSubFS(assetsFS, "assets")
	if fsys == nil {
		e.Logger.Fatal("failed to create sub FS")
	}
	// 2. "/assets" へのアクセスを、切り出したファイルシステムに紐付ける
	e.StaticFS("/assets", fsys)

	// タブのルーティング
	e.GET("/", handleAccountsRequest)
	e.GET("/accounts", handleAccountsRequest)
	e.GET("/pnl", handlePNLRequest)
	e.GET("/shop", handleShopRequest)
	e.GET("/history", handleHistoryRequest)

	e.Logger.Fatal(e.Start(":8080"))
}

func handleAccountsRequest(c echo.Context) error {
	// htmxからのリクエストかどうかで、返す外枠のコンポーネントを切り替える
	isHxRequest := c.Request().Header.Get("HX-Request") == "true"
	today := time.Now()
	currentYear := today.Year()
	currentMonth := int(today.Month())
	args := views.AccountArgs{
		CurrentYear:  currentYear,
		CurrentMonth: currentMonth,
	}
	content := views.AccountContent(args)

	if isHxRequest {
		// htmx経由: コンテンツ部分＋OOB更新用HTML
		return render(c, http.StatusOK, views.AccountPart(args, content))
	}

	// ブラウザ直アクセス: ページ全体
	return render(c, http.StatusOK, views.AccountFull(args, content))
}

func handlePNLRequest(c echo.Context) error {
	// htmxからのリクエストかどうかで、返す外枠のコンポーネントを切り替える
	isHxRequest := c.Request().Header.Get("HX-Request") == "true"
	today := time.Now()
	currentYear := today.Year()
	currentMonth := int(today.Month())
	args := views.PNLArgs{
		CurrentYear:  currentYear,
		CurrentMonth: currentMonth,
	}
	content := views.PNLContent(args)

	if isHxRequest {
		// htmx経由: コンテンツ部分＋OOB更新用HTML
		return render(c, http.StatusOK, views.PNLPart(args, content))
	}

	// ブラウザ直アクセス: ページ全体
	return render(c, http.StatusOK, views.PNLFull(args, content))
}

func handleShopRequest(c echo.Context) error {
	// htmxからのリクエストかどうかで、返す外枠のコンポーネントを切り替える
	isHxRequest := c.Request().Header.Get("HX-Request") == "true"
	today := time.Now()
	currentYear := today.Year()
	currentMonth := int(today.Month())
	args := views.ShopArgs{
		CurrentYear:  currentYear,
		CurrentMonth: currentMonth,
	}
	content := views.ShopContent(args)

	if isHxRequest {
		// htmx経由: コンテンツ部分＋OOB更新用HTML
		return render(c, http.StatusOK, views.ShopPart(args, content))
	}

	// ブラウザ直アクセス: ページ全体
	return render(c, http.StatusOK, views.ShopFull(args, content))
}

func handleHistoryRequest(c echo.Context) error {
	// htmxからのリクエストかどうかで、返す外枠のコンポーネントを切り替える
	isHxRequest := c.Request().Header.Get("HX-Request") == "true"
	today := time.Now()
	currentYear := today.Year()
	currentMonth := int(today.Month())
	args := views.HistoryArgs{
		CurrentYear:  currentYear,
		CurrentMonth: currentMonth,
	}
	content := views.HistoryContent(args)

	if isHxRequest {
		// htmx経由: コンテンツ部分＋OOB更新用HTML
		return render(c, http.StatusOK, views.HistoryPart(args, content))
	}

	// ブラウザ直アクセス: ページ全体
	return render(c, http.StatusOK, views.HistoryFull(args, content))
}
