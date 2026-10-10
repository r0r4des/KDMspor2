# KDMspor

KDM doğrulama ve klasörleme uygulaması. Hiçbir salon, sertifika ya da festival
gömülü değil; her şey uygulamanın içinden kurulur.

## Ne yapar

1. **Salonlar.** Sunucuların PEM/CRT/DER sertifikalarını bırakırsın. Uygulama
   ortak addan cihaz numarasını çıkarır, aynı cihazın sertifikalarını tek
   salonda birleştirir, zincir dosyalarındaki ara ve kök CA'ları eler, SMS ve
   log signer sertifikalarını "KDM alıcısı değil" diye işaretler.
2. **Seans listesi.** `tarih,saat,salon,film,sure_dk,dcp_adi` biçiminde bir CSV.
   Listedeki salon adı tanımlı değilse hangi salona karşılık geldiğini seçersin,
   bir kere seçtiğin eşleşme kaydedilir.
3. **CPL listesi** (opsiyonel). DCP'nin içindeki `CPL_*.xml` dosyalarını
   bırakırsın; uuid, başlık ve gerçek süre frame sayısından hesaplanır. Bu adım
   olmadan "doğru kopya" kontrolü yapılamaz.
4. **KDM dosyaları.** Zip'leri bırakırsın, iç içe olanları açar.

Sonra her seans için şunu söyler: doğru salona mı kesilmiş, pencere seansı
kapsıyor mu, doğru kopyaya mı kesilmiş, ne kadar pay var.

## Ayarlar

Saat dilimi ve üç eşik (en az ön pay, en az arka pay, süresiz filmde yeterli
pay) değiştirilebilir. Bütün kurulum tek bir JSON dosyasına yazılıp ekibe
dağıtılabilir; karşı taraf dosyayı yükleyince aynı kurulumla başlar.

## Masaüstüne özel

- **Klasöre yaz.** Zip indirip açmak yok; seçtiğin yere
  `KDM/Film/Salon/dosya` düzeniyle doğrudan yazar, eski sunucuya kesilenleri
  `Eski` alt klasörüne koyar, yanına `_ozet.csv` bırakır.
- **İzlenen klasör.** Bir klasör seçersin; oraya KDM zip'i düştüğünde uygulama
  kendiliğinden okur ve kontrol eder.

## Kurulum dosyasını üretmek

### GitHub üzerinden (bilgisayarına bir şey kurmadan)

1. Bu klasörün **içindekileri** bir GitHub deposuna yükle. `.github` klasörü
   noktayla başladığı için Finder ve Gezgin onu gizler; sürükleyip bırakmak
   yerine depoda **Add file → Create new file** deyip dosya adı kutusuna
   `.github/workflows/build.yml` yazmak daha güvenli.
2. Releases → Create a new release → tag `v1.0.0` → Create new tag on publish →
   Publish release.
3. Actions'ta dört iş başlar (Windows, macOS ARM, macOS Intel, Linux).
   15-25 dakika sonra Releases altında kurulum dosyaları birikir:

```
KDMspor_1.0.0_x64-setup.exe      Windows
KDMspor_1.0.0_x64_en-US.msi      Windows
KDMspor_1.0.0_aarch64.dmg        macOS, Apple Silicon
KDMspor_1.0.0_x64.dmg            macOS, Intel
kdmspor_1.0.0_amd64.AppImage     Linux
```

### Kendi bilgisayarında

[Node 20+](https://nodejs.org) ve [Rust](https://rustup.rs) gerekir. Windows'ta
ayrıca "Visual Studio Build Tools / Desktop development with C++", macOS'ta
`xcode-select --install`.

```
npm install
npm run build
```

Kurulum dosyası `src-tauri/target/release/bundle` altına çıkar. Geliştirirken
`npm run dev` yeterli.

## İmzalama

İmzasız kurulumda Windows "bilinmeyen yayıncı", macOS "geliştirici
doğrulanamadı" der. Uygulama yine çalışır; macOS'ta ilk açışta sağ tık → Aç.
Uyarıyı kaldırmak için ücretli kod imzalama sertifikası gerekir.

## Klasör yapısı

```
src/index.html    uygulamanın tamamı (tek dosya, tarayıcıda da açılır)
src/desktop.js    masaüstü köprüsü: kaydetme, klasöre yazma, klasör izleme
src/vendor/       jszip, forge, exceljs (projeye dahil, ağ gerekmez)
src-tauri/src/    Rust tarafı
.github/workflows kurulum dosyalarını üreten derleme ayarı
```

`src/index.html` tek başına da çalışır: çift tıklayıp tarayıcıda açarsan
klasöre yazma ve izleme dışındaki her şey aynı şekilde çalışır.

## Sürüm çıkarma

`package.json` ve `src-tauri/tauri.conf.json` içindeki `version` alanlarını
yükselt, yeni etiket at.
