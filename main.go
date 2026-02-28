package main

import (
	//"context"
	"embed"
	"fmt"
	"net/http"
	"strconv"
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
	e.GET("/accounts/:type", handleAccountsRequest)
	e.GET("/pnl/:year/:month", handlePNLRequest)
	e.GET("/shop/:year/:month", handleShopRequest)
	e.GET("/history", handleHistoryRequest)
	e.GET("/history/:account", handleHistoryRequest)

	e.Logger.Fatal(e.Start(":8080"))
}

func handleAccountsRequest(c echo.Context) error {
	// htmxからのリクエストかどうかで、返す外枠のコンポーネントを切り替える
	isHxRequest := c.Request().Header.Get("HX-Request") == "true"
	type_ := c.Param("type")
	invalidType := !(type_ == "all" || type_ == "asset" || type_ == "liability")
	if invalidType {
		type_ = "all"
	}

	today := time.Now()
	currentYear := today.Year()
	currentMonth := int(today.Month())
	args := views.AccountArgs{
		Type:         type_,
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
	yearStr := c.Param("year")
	monthStr := c.Param("month")

	// 1. 数値に変換できるかチェック
	year, errY := strconv.Atoi(yearStr)
	month, errM := strconv.Atoi(monthStr)

	// 2. ロジックチェック（2000年以降、1〜12月など）
	validationFailed := errY != nil || errM != nil || month < 1 || month > 12
	if validationFailed {
		year = currentYear
		month = currentMonth
	}

	args := views.PNLArgs{
		Year:         year,
		Month:        month,
		CurrentYear:  currentYear,
		CurrentMonth: currentMonth,
	}
	content := views.PNLContent(args)

	if isHxRequest {
		part := views.PNLPart(args, content)
		if validationFailed {
			notification := views.PNLError("入力された日付が不正です！")
			return render(c, http.StatusUnprocessableEntity, templ.Join(part, notification))
		} else {
			// htmx経由: コンテンツ部分＋OOB更新用HTML
			return render(c, http.StatusOK, part)
		}
	}

	// ブラウザ直アクセス: ページ全体
	if validationFailed {
		return c.String(http.StatusBadRequest, "Invalid Date")
	} else {
		return render(c, http.StatusOK, views.PNLFull(args, content))
	}
}

func handleShopRequest(c echo.Context) error {
	// htmxからのリクエストかどうかで、返す外枠のコンポーネントを切り替える
	isHxRequest := c.Request().Header.Get("HX-Request") == "true"
	today := time.Now()
	currentYear := today.Year()
	currentMonth := int(today.Month())
	yearStr := c.Param("year")
	monthStr := c.Param("month")

	// 1. 数値に変換できるかチェック
	year, errY := strconv.Atoi(yearStr)
	month, errM := strconv.Atoi(monthStr)

	// 2. ロジックチェック（2000年以降、1〜12月など）
	validationFailed := errY != nil || errM != nil || month < 1 || month > 12
	if validationFailed {
		year = currentYear
		month = currentMonth
	}

	args := views.ShopArgs{
		Year:         year,
		Month:        month,
		CurrentYear:  currentYear,
		CurrentMonth: currentMonth,
	}
	content := views.ShopContent(args)

	if isHxRequest {
		part := views.ShopPart(args, content)
		if validationFailed {
			notification := views.ShopError("入力された日付が不正です！")
			return render(c, http.StatusUnprocessableEntity, templ.Join(part, notification))
		} else {
			// htmx経由: コンテンツ部分＋OOB更新用HTML
			return render(c, http.StatusOK, part)
		}
	}

	// ブラウザ直アクセス: ページ全体
	if validationFailed {
		return c.String(http.StatusBadRequest, "Invalid Date")
	} else {
		return render(c, http.StatusOK, views.ShopFull(args, content))
	}
}

func handleHistoryRequest(c echo.Context) error {
	// htmxからのリクエストかどうかで、返す外枠のコンポーネントを切り替える
	isHxRequest := c.Request().Header.Get("HX-Request") == "true"
	today := time.Now()
	currentYear := today.Year()
	currentMonth := int(today.Month())

	account := c.Param("account")
	from_ := c.QueryParam("from")
	to_ := c.QueryParam("to")
	monthStr := c.QueryParam("month")

	// 1. 数値に変換できるかチェック
	var year, month int
	_, err := fmt.Sscanf(monthStr, "%d-%d", &year, &month)

	// 2. ロジックチェック（2000年以降、1〜12月など）
	validationFailed := err != nil || month < 1 || month > 12
	if validationFailed {
		year = 0
		month = 0
	}

	args := views.HistoryArgs{
		Account:      account,
		AccountName: "アカウント名",
		CurrentYear:  currentYear,
		CurrentMonth: currentMonth,
		Year:         year,
		Month:        month,
		From:         from_,
		To:           to_,
	}
	content := views.HistoryContent(args)

	if isHxRequest {
		// htmx経由: コンテンツ部分＋OOB更新用HTML
		return render(c, http.StatusOK, views.HistoryPart(args, content))
	}

	// ブラウザ直アクセス: ページ全体
	return render(c, http.StatusOK, views.HistoryFull(args, content))
}
