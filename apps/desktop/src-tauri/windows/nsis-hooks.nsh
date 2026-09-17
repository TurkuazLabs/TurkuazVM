; 📄 Dosya Yolu: /turkuazvm/apps/desktop/src-tauri/windows/nsis-hooks.nsh
; 📌 Amac: TurkuazVM NSIS installer icin Tauri/NSIS upstream'de eksik kalan Turkce MultiUser metinlerini tamamlar
; 📌 Modul - NSIS Hook
; Version: 0.41.5
; Aciklama: installMode=both secim sayfasinda Ingilizce fallback olusmasini engeller; Tauri ana installer template'i fork edilmez
; Bagimli Oldugu Katman: Tool | View | CI/CD

; MultiUser.nsh bu anahtarlari MUI_LANGUAGE yuklenirken okur. Turkish ilk dil oldugu
; icin bu tanimlar Turkce tabloya yazilir ve ardindan NSIS tarafindan undef edilir;
; sonraki English dili kendi upstream metinlerini kullanmaya devam eder.
!define MULTIUSER_TEXT_INSTALLMODE_TITLE "Kurulum türünü seçin"
!define MULTIUSER_TEXT_INSTALLMODE_SUBTITLE "TurkuazVM'in kimler için kurulacağını seçin."
!define MULTIUSER_INNERTEXT_INSTALLMODE_TOP "TurkuazVM'i yalnızca bu kullanıcı için veya bu bilgisayardaki tüm kullanıcılar için kurabilirsiniz."
!define MULTIUSER_INNERTEXT_INSTALLMODE_ALLUSERS "Bu bilgisayardaki tüm kullanıcılar için kur"
!define MULTIUSER_INNERTEXT_INSTALLMODE_CURRENTUSER "Yalnızca benim için kur"
