use std::fs;
use std::time::SystemTime;
use std::path::{Path, PathBuf};

/// Test helper that creates a disposable file tree and cleans it up on drop.
/// You can add dirs/files, or copy an entire fixture tree (e.g. tests/test_project) into it.
#[cfg(test)]
pub struct FsTree {
    level: u8,
    root: PathBuf,
    created: Vec<PathBuf>,
}


/// Test Project Intializer Templates Function 
/// methods: 
/// ```
/// let mut tree = FsTree::new("test_project")?;
/// tree.create_structure(true)?; // for C project or else false for C++ project
/// tree.create_realistic_project(false)?; // for C++ realistic project or true for C realistic project
/// 
/// ```

#[cfg(test)]
impl FsTree {
    /// Create (or recreate) the root directory for this test tree.
    ///
    /// Safety: this always creates a unique directory under the OS temp folder,
    /// so tests can't accidentally delete your real project files.
    pub fn new(name: &Path) -> Result<Self, String> {
        println!("[FsTree] Creating new test tree...");
        
        let name = name.to_string_lossy();
        let safe_name: String = name.chars().map(|c| if c.is_ascii_alphanumeric() || c == '-' || c == '_' { c } else { '_' }).collect();
        let ts = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        println!("[FsTree]   Safe name: {}", safe_name);
        println!("current working directory is  : {}", std::env::current_dir().unwrap().display());

        let root = std::env::temp_dir()
            .join(format!("ion_test_trees_{}_{}_{}", safe_name, std::process::id(), ts));

        println!("[FsTree] Root directory: {}", root.display());
        fs::create_dir_all(&root).map_err(|e| e.to_string())?;
        println!("[FsTree] ✓ Root directory created successfully");
        
        Ok(Self {
            level: 0,
            root,
            created: Vec::new(),
        })
    }

    /// Returns the root path for convenience in assertions.
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// Create a minimal C/C++ style project layout and seed main file.
    pub fn create_minimal_project(&mut self, lang_c: bool) -> Result<(), String> {
        if self.level > 1 {
            return Err("FsTree: Bigger Version Exists".to_string());
        }
        let lang = if lang_c { "C" } else { "C++" };
        println!("[FsTree] Creating minimal {} project structure...", lang);
        
        let src = self.root.join("src");
        let include = self.root.join("include");
        let build = self.root.join("build");
        let main_file = src.join(if lang_c { "main.c" } else { "main.cpp" });

        println!("[FsTree]   Creating dir: {}", src.display());
        fs::create_dir_all(&src).map_err(|e| e.to_string())?;
        println!("[FsTree]   Creating dir: {}", include.display());
        fs::create_dir_all(&include).map_err(|e| e.to_string())?;
        println!("[FsTree]   Creating dir: {}", build.display());
        fs::create_dir_all(&build).map_err(|e| e.to_string())?;

        let template = if lang_c { CTEMPLATE_MAIN } else { CPPTEMPLATE_MAIN };
        println!("[FsTree]   Creating file: {}", main_file.display());
        fs::write(&main_file, template).map_err(|e| e.to_string())?;
        
        println!("[FsTree] ✓ Minimal structure created");
        self.level = 1 ;
        Ok(())
    }

    /// Create a full C/C++ project with 5 source files and 5 header files.
    /// All files are connected and usable from main.
    pub fn create_full_project(&mut self, lang_c: bool) -> Result<(), String> {
       
        let lang = if lang_c { "C" } else { "C++" };
        println!("[FsTree] Creating full {} project...", lang);
        
        let src = self.root.join("src");
        let include = self.root.join("include");
        let build = self.root.join("build");
        let tests = self.root.join("tests");

        // Create directory structure
        println!("[FsTree]   Creating directories: src, include, build, tests");
        fs::create_dir_all(&src).map_err(|e| e.to_string())?;
        fs::create_dir_all(&include).map_err(|e| e.to_string())?;
        fs::create_dir_all(&build).map_err(|e| e.to_string())?;
        fs::create_dir_all(&tests).map_err(|e| e.to_string())?;

        let (templates, main_template) = if lang_c {
            // For C language
            (CTEMPLATES, CTEMPLATE_MAIN)
        } else {
            // For C++ language
            (CPPTEMPLATES, CPPMAIN_TEMPLATE)
        };

        // Create all template files
        println!("[FsTree]   Creating {} template files...", templates.len());
        for (rel_path, content) in templates {
            let path = self.root.join(rel_path);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).map_err(|e| e.to_string())?;
            }
            println!("[FsTree]     + {}", rel_path);
            fs::write(&path, content).map_err(|e| e.to_string())?;
        }

        // Create main file
        let main_file = src.join(if lang_c { "main.c" } else { "main.cpp" });
        println!("[FsTree]   Creating main file: {}", main_file.display());
        fs::write(&main_file, main_template).map_err(|e| e.to_string())?;

        println!("[FsTree] ✓ Full project created");
        self.level = 3 ;
        Ok(())
    }

    /// Create a test project fixture with proper C++ project structure.
    /// Headers go in include/, sources go in src/.
    pub fn create_test_project_fixture(&mut self) -> Result<(), String> {

        if self.level > 2 {
            return Err("FsTree: Bigger Version Exists".to_string());
        }

        println!("[FsTree] Creating test project fixture...");
        
        // Create directory structure
        println!("[FsTree]   Creating directories...");
        self.create_dir("src")?;
        self.create_dir("include")?;
        self.create_dir("build")?;

        // Create header files in include/
        println!("[FsTree]   Creating header files in include/...");
        self.create_file("include/logger.h", FIXTURE_LOGGER_H.as_bytes())?;
        self.create_file("include/math_utils.h", FIXTURE_MATH_UTILS_H.as_bytes())?;
        self.create_file("include/utils.h", FIXTURE_UTILS_H.as_bytes())?;

        // Create source files in src/
        println!("[FsTree]   Creating source files in src/...");
        self.create_file("src/main.cpp", FIXTURE_MAIN_CPP.as_bytes())?;
        self.create_file("src/logger.cpp", FIXTURE_LOGGER_CPP.as_bytes())?;
        self.create_file("src/math_utils.cpp", FIXTURE_MATH_UTILS_CPP.as_bytes())?;
        self.create_file("src/utils.cpp", FIXTURE_UTILS_CPP.as_bytes())?;

        println!("[FsTree] ✓ Test project fixture created");
        self.level = 2 ;
        Ok(())
    }

    /// Create a directory relative to the root and return its path.
    pub fn create_dir(&mut self, rel: impl AsRef<Path>) -> Result<PathBuf, String> {
        let rel_path = rel.as_ref();
        let dir = self.root.join(rel_path);
        println!("[FsTree]     + dir: {}", rel_path.display());
        fs::create_dir_all(&dir).map_err(|e| {
            println!("[FsTree]     ✗ Failed to create dir: {}", e);
            e.to_string()
        })?;
        self.created.push(dir.clone());
        Ok(dir)
    }

    /// Create a file with content relative to the root and return its path.
    pub fn create_file(
        &mut self,
        rel: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> Result<PathBuf, String> {
        let rel_path = rel.as_ref();
        let file = self.root.join(rel_path);
        let size = contents.as_ref().len();
        println!("[FsTree]     + file: {} ({} bytes)", rel_path.display(), size);
        if let Some(parent) = file.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                println!("[FsTree]     ✗ Failed to create parent dir: {}", e);
                e.to_string()
            })?;
        }
        fs::write(&file, contents).map_err(|e| {
            println!("[FsTree]     ✗ Failed to write file: {}", e);
            e.to_string()
        })?;
        self.created.push(file.clone());
        Ok(file)
    }
}

#[cfg(test)]
impl Drop for FsTree {

    fn drop(&mut self) {
        println!("[FsTree] Cleaning up test tree: {}", self.root.display());
        println!("[FsTree]   Total items tracked: {}", self.created.len());
        // Best-effort cleanup; ignore errors to avoid test panics during drop.
        match fs::remove_dir_all(&self.root) {
            Ok(_) => println!("[FsTree] ✓ Cleanup complete"),
            Err(e) => println!("[FsTree] ⚠ Cleanup failed (non-fatal): {}", e),
        }
    }
    
}

// ===================== C++ TEMPLATES =====================

#[cfg(test)]
const CPPTEMPLATE_MAIN: &str = r#"#include <iostream>
#include "calculator.hpp"
#include "string_utils.hpp"
#include "data_processor.hpp"
#include "logger.hpp"
#include "validation.hpp"

int main() {
    Logger::init();
    Logger::log("Starting application...");
    
    // Test Calculator
    std::cout << "Calculator tests:" << std::endl;
    std::cout << "5 + 3 = " << Calculator::add(5, 3) << std::endl;
    std::cout << "10 * 2 = " << Calculator::multiply(10, 2) << std::endl;
    std::cout << "Factorial of 5 = " << Calculator::factorial(5) << std::endl;
    std::cout << "Is 17 prime? " << (Calculator::isPrime(17) ? "Yes" : "No") << std::endl;
    
    // Test StringUtils
    std::string testStr = "Hello, World!";
    std::cout << "\nStringUtils tests:" << std::endl;
    std::cout << "Original: " << testStr << std::endl;
    std::cout << "Reversed: " << StringUtils::reverse(testStr) << std::endl;
    std::cout << "Uppercase: " << StringUtils::toUpper(testStr) << std::endl;
    std::cout << "Lowercase: " << StringUtils::toLower(testStr) << std::endl;
    std::cout << "Has 'World'? " << (StringUtils::contains(testStr, "World") ? "Yes" : "No") << std::endl;
    
    // Test DataProcessor
    DataProcessor processor;
    std::vector<int> data = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    std::cout << "\nDataProcessor tests:" << std::endl;
    std::cout << "Sum: " << processor.sum(data) << std::endl;
    std::cout << "Average: " << processor.average(data) << std::endl;
    std::cout << "Max: " << processor.max(data) << std::endl;
    std::cout << "Min: " << processor.min(data) << std::endl;
    
    auto filtered = processor.filterEven(data);
    std::cout << "Even numbers count: " << filtered.size() << std::endl;
    
    // Test Validation
    std::cout << "\nValidation tests:" << std::endl;
    std::cout << "Is 'test@email.com' valid email? " 
              << (Validation::isValidEmail("test@email.com") ? "Yes" : "No") << std::endl;
    std::cout << "Is '123-45-6789' valid SSN? " 
              << (Validation::isValidSSN("123-45-6789") ? "Yes" : "No") << std::endl;
    std::cout << "Is '1234567890' valid phone? " 
              << (Validation::isValidPhone("1234567890") ? "Yes" : "No") << std::endl;
    
    // Process complex data
    std::cout << "\nProcessing complex data..." << std::endl;
    auto processed = processor.process(data, [](int x) { return x * x; });
    std::cout << "Squared values processed." << std::endl;
    
    Logger::log("Application completed successfully");
    Logger::cleanup();
    
    return 0;
}
"#;

#[cfg(test)]
const CALCULATOR_HPP: &str = r#"#ifndef CALCULATOR_HPP
#define CALCULATOR_HPP

#include <vector>
#include <cmath>

class Calculator {
public:
    // Basic arithmetic operations
    static int add(int a, int b);
    static int subtract(int a, int b);
    static int multiply(int a, int b);
    static double divide(int a, int b);
    
    // Advanced mathematical operations
    static int factorial(int n);
    static bool isPrime(int n);
    static int gcd(int a, int b);
    static int lcm(int a, int b);
    static double power(double base, double exponent);
    
    // Statistical functions
    static double mean(const std::vector<int>& numbers);
    static double standardDeviation(const std::vector<int>& numbers);
    static int mode(const std::vector<int>& numbers);
    
    // Financial calculations
    static double compoundInterest(double principal, double rate, int time);
    static double simpleInterest(double principal, double rate, int time);
    static double presentValue(double futureValue, double rate, int periods);
    
    // Utility functions
    static bool isEven(int n);
    static bool isOdd(int n);
    static int absolute(int n);
    static double squareRoot(double n);
    
private:
    static bool checkDivisionByZero(int b);
};

#endif // CALCULATOR_HPP
"#;

#[cfg(test)]
const CALCULATOR_CPP: &str = r#"#include "calculator.hpp"
#include <stdexcept>
#include <algorithm>
#include <map>
#include <cmath>

int Calculator::add(int a, int b) {
    return a + b;
}

int Calculator::subtract(int a, int b) {
    return a - b;
}

int Calculator::multiply(int a, int b) {
    return a * b;
}

double Calculator::divide(int a, int b) {
    if (checkDivisionByZero(b)) {
        throw std::runtime_error("Division by zero error");
    }
    return static_cast<double>(a) / b;
}

int Calculator::factorial(int n) {
    if (n < 0) {
        throw std::invalid_argument("Factorial of negative number is undefined");
    }
    int result = 1;
    for (int i = 2; i <= n; ++i) {
        result *= i;
    }
    return result;
}

bool Calculator::isPrime(int n) {
    if (n <= 1) return false;
    if (n <= 3) return true;
    if (n % 2 == 0 || n % 3 == 0) return false;
    
    for (int i = 5; i * i <= n; i += 6) {
        if (n % i == 0 || n % (i + 2) == 0) {
            return false;
        }
    }
    return true;
}

int Calculator::gcd(int a, int b) {
    while (b != 0) {
        int temp = b;
        b = a % b;
        a = temp;
    }
    return a;
}

int Calculator::lcm(int a, int b) {
    return (a / gcd(a, b)) * b;
}

double Calculator::power(double base, double exponent) {
    return std::pow(base, exponent);
}

double Calculator::mean(const std::vector<int>& numbers) {
    if (numbers.empty()) {
        return 0.0;
    }
    double sum = 0.0;
    for (int num : numbers) {
        sum += num;
    }
    return sum / numbers.size();
}

double Calculator::standardDeviation(const std::vector<int>& numbers) {
    if (numbers.size() <= 1) {
        return 0.0;
    }
    double m = mean(numbers);
    double sum = 0.0;
    for (int num : numbers) {
        double diff = num - m;
        sum += diff * diff;
    }
    return std::sqrt(sum / (numbers.size() - 1));
}

int Calculator::mode(const std::vector<int>& numbers) {
    if (numbers.empty()) {
        throw std::invalid_argument("Cannot find mode of empty vector");
    }
    
    std::map<int, int> frequency;
    for (int num : numbers) {
        frequency[num]++;
    }
    
    auto maxElement = std::max_element(
        frequency.begin(),
        frequency.end(),
        [](const auto& a, const auto& b) {
            return a.second < b.second;
        }
    );
    
    return maxElement->first;
}

double Calculator::compoundInterest(double principal, double rate, int time) {
    return principal * std::pow(1 + rate, time);
}

double Calculator::simpleInterest(double principal, double rate, int time) {
    return principal * (1 + rate * time);
}

double Calculator::presentValue(double futureValue, double rate, int periods) {
    return futureValue / std::pow(1 + rate, periods);
}

bool Calculator::isEven(int n) {
    return n % 2 == 0;
}

bool Calculator::isOdd(int n) {
    return n % 2 != 0;
}

int Calculator::absolute(int n) {
    return n < 0 ? -n : n;
}

double Calculator::squareRoot(double n) {
    if (n < 0) {
        throw std::invalid_argument("Cannot calculate square root of negative number");
    }
    return std::sqrt(n);
}

bool Calculator::checkDivisionByZero(int b) {
    return b == 0;
}
"#;

#[cfg(test)]
const STRING_UTILS_HPP: &str = r#"#ifndef STRING_UTILS_HPP
#define STRING_UTILS_HPP

#include <string>
#include <vector>
#include <algorithm>
#include <cctype>
#include <sstream>
#include <iomanip>

class StringUtils {
public:
    // Basic string operations
    static std::string reverse(const std::string& str);
    static std::string toUpper(const std::string& str);
    static std::string toLower(const std::string& str);
    static std::string trim(const std::string& str);
    
    // String checks
    static bool startsWith(const std::string& str, const std::string& prefix);
    static bool endsWith(const std::string& str, const std::string& suffix);
    static bool contains(const std::string& str, const std::string& substring);
    static bool isEmpty(const std::string& str);
    
    // String manipulation
    static std::string replace(const std::string& str, 
                               const std::string& from, 
                               const std::string& to);
    static std::string padLeft(const std::string& str, 
                               char paddingChar, 
                               size_t totalLength);
    static std::string padRight(const std::string& str, 
                                char paddingChar, 
                                size_t totalLength);
    
    // String splitting and joining
    static std::vector<std::string> split(const std::string& str, 
                                          char delimiter);
    static std::string join(const std::vector<std::string>& strings, 
                            const std::string& delimiter);
    
    // Character operations
    static int countOccurrences(const std::string& str, char ch);
    static std::string removeCharacter(const std::string& str, char ch);
    
    // Case conversion utilities
    static std::string toCamelCase(const std::string& str);
    static std::string toSnakeCase(const std::string& str);
    
    // Validation utilities
    static bool isNumeric(const std::string& str);
    static bool isAlpha(const std::string& str);
    static bool isAlphaNumeric(const std::string& str);
    
    // Encoding/decoding
    static std::string urlEncode(const std::string& str);
    static std::string htmlEscape(const std::string& str);
};

#endif // STRING_UTILS_HPP
"#;

#[cfg(test)]
const STRING_UTILS_CPP: &str = r#"#include "string_utils.hpp"
#include <stdexcept>

std::string StringUtils::reverse(const std::string& str) {
    return std::string(str.rbegin(), str.rend());
}

std::string StringUtils::toUpper(const std::string& str) {
    std::string result = str;
    std::transform(result.begin(), result.end(), result.begin(),
                   [](unsigned char c) { return std::toupper(c); });
    return result;
}

std::string StringUtils::toLower(const std::string& str) {
    std::string result = str;
    std::transform(result.begin(), result.end(), result.begin(),
                   [](unsigned char c) { return std::tolower(c); });
    return result;
}

std::string StringUtils::trim(const std::string& str) {
    size_t first = str.find_first_not_of(" \t\n\r");
    if (first == std::string::npos) {
        return "";
    }
    size_t last = str.find_last_not_of(" \t\n\r");
    return str.substr(first, (last - first + 1));
}

bool StringUtils::startsWith(const std::string& str, const std::string& prefix) {
    if (prefix.length() > str.length()) {
        return false;
    }
    return str.compare(0, prefix.length(), prefix) == 0;
}

bool StringUtils::endsWith(const std::string& str, const std::string& suffix) {
    if (suffix.length() > str.length()) {
        return false;
    }
    return str.compare(str.length() - suffix.length(), suffix.length(), suffix) == 0;
}

bool StringUtils::contains(const std::string& str, const std::string& substring) {
    return str.find(substring) != std::string::npos;
}

bool StringUtils::isEmpty(const std::string& str) {
    return str.empty();
}

std::string StringUtils::replace(const std::string& str, 
                                 const std::string& from, 
                                 const std::string& to) {
    std::string result = str;
    size_t start_pos = 0;
    while ((start_pos = result.find(from, start_pos)) != std::string::npos) {
        result.replace(start_pos, from.length(), to);
        start_pos += to.length();
    }
    return result;
}

std::string StringUtils::padLeft(const std::string& str, 
                                 char paddingChar, 
                                 size_t totalLength) {
    if (str.length() >= totalLength) {
        return str;
    }
    return std::string(totalLength - str.length(), paddingChar) + str;
}

std::string StringUtils::padRight(const std::string& str, 
                                  char paddingChar, 
                                  size_t totalLength) {
    if (str.length() >= totalLength) {
        return str;
    }
    return str + std::string(totalLength - str.length(), paddingChar);
}

std::vector<std::string> StringUtils::split(const std::string& str, 
                                            char delimiter) {
    std::vector<std::string> tokens;
    std::string token;
    std::istringstream tokenStream(str);
    
    while (std::getline(tokenStream, token, delimiter)) {
        if (!token.empty()) {
            tokens.push_back(token);
        }
    }
    
    return tokens;
}

std::string StringUtils::join(const std::vector<std::string>& strings, 
                              const std::string& delimiter) {
    std::ostringstream oss;
    for (size_t i = 0; i < strings.size(); ++i) {
        if (i != 0) {
            oss << delimiter;
        }
        oss << strings[i];
    }
    return oss.str();
}

int StringUtils::countOccurrences(const std::string& str, char ch) {
    return std::count(str.begin(), str.end(), ch);
}

std::string StringUtils::removeCharacter(const std::string& str, char ch) {
    std::string result;
    std::copy_if(str.begin(), str.end(), std::back_inserter(result),
                 [ch](char c) { return c != ch; });
    return result;
}

std::string StringUtils::toCamelCase(const std::string& str) {
    std::string result;
    bool capitalizeNext = false;
    
    for (char c : str) {
        if (c == '_' || c == ' ') {
            capitalizeNext = true;
        } else if (capitalizeNext) {
            result += std::toupper(c);
            capitalizeNext = false;
        } else {
            result += c;
        }
    }
    
    return result;
}

std::string StringUtils::toSnakeCase(const std::string& str) {
    std::string result;
    
    for (char c : str) {
        if (std::isupper(c)) {
            if (!result.empty()) {
                result += '_';
            }
            result += std::tolower(c);
        } else if (c == ' ') {
            result += '_';
        } else {
            result += c;
        }
    }
    
    return result;
}

bool StringUtils::isNumeric(const std::string& str) {
    if (str.empty()) {
        return false;
    }
    
    size_t start = 0;
    if (str[0] == '-') {
        start = 1;
        if (str.length() == 1) {
            return false;
        }
    }
    
    bool hasDecimal = false;
    for (size_t i = start; i < str.length(); ++i) {
        if (str[i] == '.') {
            if (hasDecimal) {
                return false;
            }
            hasDecimal = true;
        } else if (!std::isdigit(str[i])) {
            return false;
        }
    }
    
    return true;
}

bool StringUtils::isAlpha(const std::string& str) {
    return std::all_of(str.begin(), str.end(),
                       [](unsigned char c) { return std::isalpha(c); });
}

bool StringUtils::isAlphaNumeric(const std::string& str) {
    return std::all_of(str.begin(), str.end(),
                       [](unsigned char c) { return std::isalnum(c); });
}

std::string StringUtils::urlEncode(const std::string& str) {
    std::ostringstream encoded;
    encoded << std::hex << std::uppercase;
    
    for (char c : str) {
        if (std::isalnum(c) || c == '-' || c == '_' || c == '.' || c == '~') {
            encoded << c;
        } else {
            encoded << '%' << std::setw(2) << std::setfill('0') 
                    << static_cast<int>(static_cast<unsigned char>(c));
        }
    }
    
    return encoded.str();
}

std::string StringUtils::htmlEscape(const std::string& str) {
    std::string result;
    result.reserve(str.length());
    
    for (char c : str) {
        switch (c) {
            case '&':  result += "&amp;";  break;
            case '<':  result += "&lt;";   break;
            case '>':  result += "&gt;";   break;
            case '"':  result += "&quot;"; break;
            case '\'': result += "&#39;";  break;
            default:   result += c;        break;
        }
    }
    
    return result;
}
"#;

#[cfg(test)]
const DATA_PROCESSOR_HPP: &str = r#"#ifndef DATA_PROCESSOR_HPP
#define DATA_PROCESSOR_HPP

#include <vector>
#include <functional>
#include <algorithm>
#include <numeric>
#include <map>
#include <set>

class DataProcessor {
public:
    // Basic statistical operations
    double sum(const std::vector<int>& data);
    double average(const std::vector<int>& data);
    int max(const std::vector<int>& data);
    int min(const std::vector<int>& data);
    double median(const std::vector<int>& data);
    
    // Data transformation
    std::vector<int> filterEven(const std::vector<int>& data);
    std::vector<int> filterOdd(const std::vector<int>& data);
    std::vector<int> square(const std::vector<int>& data);
    std::vector<int> cube(const std::vector<int>& data);
    
    // Data analysis
    std::map<int, int> frequencyCount(const std::vector<int>& data);
    std::vector<int> findDuplicates(const std::vector<int>& data);
    std::vector<int> findUnique(const std::vector<int>& data);
    
    // Sorting operations
    std::vector<int> sortAscending(const std::vector<int>& data);
    std::vector<int> sortDescending(const std::vector<int>& data);
    std::vector<int> reverseOrder(const std::vector<int>& data);
    
    // Set operations
    std::vector<int> unionSets(const std::vector<int>& set1, 
                               const std::vector<int>& set2);
    std::vector<int> intersection(const std::vector<int>& set1, 
                                  const std::vector<int>& set2);
    std::vector<int> difference(const std::vector<int>& set1, 
                                const std::vector<int>& set2);
    
    // Data processing with callbacks
    template<typename Func>
    std::vector<int> process(const std::vector<int>& data, Func func);
    
    // Utility functions
    bool contains(const std::vector<int>& data, int value);
    int countOccurrences(const std::vector<int>& data, int value);
    std::vector<int> slice(const std::vector<int>& data, 
                           size_t start, 
                           size_t end);
    
private:
    void validateData(const std::vector<int>& data);
};

// Template implementation must be in header
template<typename Func>
std::vector<int> DataProcessor::process(const std::vector<int>& data, Func func) {
    std::vector<int> result;
    result.reserve(data.size());
    std::transform(data.begin(), data.end(), std::back_inserter(result), func);
    return result;
}

#endif // DATA_PROCESSOR_HPP
"#;

#[cfg(test)]
const DATA_PROCESSOR_CPP: &str = r#"#include "data_processor.hpp"
#include <stdexcept>
#include <cmath>

double DataProcessor::sum(const std::vector<int>& data) {
    validateData(data);
    return std::accumulate(data.begin(), data.end(), 0.0);
}

double DataProcessor::average(const std::vector<int>& data) {
    validateData(data);
    if (data.empty()) {
        return 0.0;
    }
    return sum(data) / data.size();
}

int DataProcessor::max(const std::vector<int>& data) {
    validateData(data);
    if (data.empty()) {
        throw std::invalid_argument("Cannot find max of empty vector");
    }
    return *std::max_element(data.begin(), data.end());
}

int DataProcessor::min(const std::vector<int>& data) {
    validateData(data);
    if (data.empty()) {
        throw std::invalid_argument("Cannot find min of empty vector");
    }
    return *std::min_element(data.begin(), data.end());
}

double DataProcessor::median(const std::vector<int>& data) {
    validateData(data);
    if (data.empty()) {
        throw std::invalid_argument("Cannot find median of empty vector");
    }
    
    std::vector<int> sorted = data;
    std::sort(sorted.begin(), sorted.end());
    
    size_t size = sorted.size();
    if (size % 2 == 0) {
        return (sorted[size/2 - 1] + sorted[size/2]) / 2.0;
    } else {
        return sorted[size/2];
    }
}

std::vector<int> DataProcessor::filterEven(const std::vector<int>& data) {
    validateData(data);
    std::vector<int> result;
    std::copy_if(data.begin(), data.end(), std::back_inserter(result),
                 [](int n) { return n % 2 == 0; });
    return result;
}

std::vector<int> DataProcessor::filterOdd(const std::vector<int>& data) {
    validateData(data);
    std::vector<int> result;
    std::copy_if(data.begin(), data.end(), std::back_inserter(result),
                 [](int n) { return n % 2 != 0; });
    return result;
}

std::vector<int> DataProcessor::square(const std::vector<int>& data) {
    validateData(data);
    return process(data, [](int x) { return x * x; });
}

std::vector<int> DataProcessor::cube(const std::vector<int>& data) {
    validateData(data);
    return process(data, [](int x) { return x * x * x; });
}

std::map<int, int> DataProcessor::frequencyCount(const std::vector<int>& data) {
    validateData(data);
    std::map<int, int> freq;
    for (int value : data) {
        freq[value]++;
    }
    return freq;
}

std::vector<int> DataProcessor::findDuplicates(const std::vector<int>& data) {
    validateData(data);
    std::vector<int> duplicates;
    std::map<int, int> freq = frequencyCount(data);
    
    for (const auto& pair : freq) {
        if (pair.second > 1) {
            duplicates.push_back(pair.first);
        }
    }
    
    return duplicates;
}

std::vector<int> DataProcessor::findUnique(const std::vector<int>& data) {
    validateData(data);
    std::set<int> uniqueSet(data.begin(), data.end());
    return std::vector<int>(uniqueSet.begin(), uniqueSet.end());
}

std::vector<int> DataProcessor::sortAscending(const std::vector<int>& data) {
    validateData(data);
    std::vector<int> result = data;
    std::sort(result.begin(), result.end());
    return result;
}

std::vector<int> DataProcessor::sortDescending(const std::vector<int>& data) {
    validateData(data);
    std::vector<int> result = data;
    std::sort(result.begin(), result.end(), std::greater<int>());
    return result;
}

std::vector<int> DataProcessor::reverseOrder(const std::vector<int>& data) {
    validateData(data);
    std::vector<int> result = data;
    std::reverse(result.begin(), result.end());
    return result;
}

std::vector<int> DataProcessor::unionSets(const std::vector<int>& set1, 
                                          const std::vector<int>& set2) {
    validateData(set1);
    validateData(set2);
    std::set<int> unionSet(set1.begin(), set1.end());
    unionSet.insert(set2.begin(), set2.end());
    return std::vector<int>(unionSet.begin(), unionSet.end());
}

std::vector<int> DataProcessor::intersection(const std::vector<int>& set1, 
                                             const std::vector<int>& set2) {
    validateData(set1);
    validateData(set2);
    std::vector<int> result;
    std::set_intersection(set1.begin(), set1.end(),
                          set2.begin(), set2.end(),
                          std::back_inserter(result));
    return result;
}

std::vector<int> DataProcessor::difference(const std::vector<int>& set1, 
                                           const std::vector<int>& set2) {
    validateData(set1);
    validateData(set2);
    std::vector<int> result;
    std::set_difference(set1.begin(), set1.end(),
                        set2.begin(), set2.end(),
                        std::back_inserter(result));
    return result;
}

bool DataProcessor::contains(const std::vector<int>& data, int value) {
    validateData(data);
    return std::find(data.begin(), data.end(), value) != data.end();
}

int DataProcessor::countOccurrences(const std::vector<int>& data, int value) {
    validateData(data);
    return std::count(data.begin(), data.end(), value);
}

std::vector<int> DataProcessor::slice(const std::vector<int>& data, 
                                      size_t start, 
                                      size_t end) {
    validateData(data);
    if (start >= data.size() || end > data.size() || start >= end) {
        throw std::out_of_range("Invalid slice range");
    }
    
    std::vector<int> result(data.begin() + start, data.begin() + end);
    return result;
}

void DataProcessor::validateData(const std::vector<int>& data) {
    // Could add more validation logic here
    // For now, just check if we can access the data
    if (data.size() > 1000000) {
        throw std::invalid_argument("Data size too large");
    }
}
"#;

#[cfg(test)]
const LOGGER_HPP: &str = r#"#ifndef LOGGER_HPP
#define LOGGER_HPP

#include <string>
#include <fstream>
#include <mutex>
#include <chrono>
#include <iomanip>
#include <iostream>
#include <sstream>

class Logger {
public:
    // Log levels
    enum class Level {
        DEBUG,
        INFO,
        WARNING,
        ERROR,
        CRITICAL
    };
    
    // Initialize and cleanup
    static void init(const std::string& logFile = "app.log");
    static void cleanup();
    
    // Logging methods
    static void log(const std::string& message, 
                    Level level = Level::INFO);
    static void debug(const std::string& message);
    static void info(const std::string& message);
    static void warning(const std::string& message);
    static void error(const std::string& message);
    static void critical(const std::string& message);
    
    // Configuration
    static void setLogLevel(Level level);
    static void enableConsoleOutput(bool enable);
    static void enableFileOutput(bool enable);
    
    // Utility methods
    static std::string getCurrentTime();
    static std::string levelToString(Level level);
    
private:
    static std::ofstream logFileStream;
    static std::mutex logMutex;
    static Level currentLevel;
    static bool consoleOutputEnabled;
    static bool fileOutputEnabled;
    static std::string logFileName;
    
    static void writeToLog(const std::string& message, Level level);
};

#endif // LOGGER_HPP
"#;

#[cfg(test)]
const LOGGER_CPP: &str = r#"#include "logger.hpp"
#include <stdexcept>

// Static member initialization
std::ofstream Logger::logFileStream;
std::mutex Logger::logMutex;
Logger::Level Logger::currentLevel = Logger::Level::INFO;
bool Logger::consoleOutputEnabled = true;
bool Logger::fileOutputEnabled = true;
std::string Logger::logFileName = "app.log";

void Logger::init(const std::string& logFile) {
    std::lock_guard<std::mutex> lock(logMutex);
    logFileName = logFile;
    
    if (fileOutputEnabled) {
        logFileStream.open(logFile, std::ios::app);
        if (!logFileStream.is_open()) {
            throw std::runtime_error("Failed to open log file: " + logFile);
        }
    }
    
    info("Logger initialized");
}

void Logger::cleanup() {
    std::lock_guard<std::mutex> lock(logMutex);
    if (logFileStream.is_open()) {
        info("Logger shutting down");
        logFileStream.close();
    }
}

void Logger::log(const std::string& message, Level level) {
    if (level < currentLevel) {
        return;
    }
    
    writeToLog(message, level);
}

void Logger::debug(const std::string& message) {
    log(message, Level::DEBUG);
}

void Logger::info(const std::string& message) {
    log(message, Level::INFO);
}

void Logger::warning(const std::string& message) {
    log(message, Level::WARNING);
}

void Logger::error(const std::string& message) {
    log(message, Level::ERROR);
}

void Logger::critical(const std::string& message) {
    log(message, Level::CRITICAL);
}

void Logger::setLogLevel(Level level) {
    std::lock_guard<std::mutex> lock(logMutex);
    currentLevel = level;
}

void Logger::enableConsoleOutput(bool enable) {
    std::lock_guard<std::mutex> lock(logMutex);
    consoleOutputEnabled = enable;
}

void Logger::enableFileOutput(bool enable) {
    std::lock_guard<std::mutex> lock(logMutex);
    fileOutputEnabled = enable;
    
    if (enable && !logFileStream.is_open()) {
        logFileStream.open(logFileName, std::ios::app);
    } else if (!enable && logFileStream.is_open()) {
        logFileStream.close();
    }
}

std::string Logger::getCurrentTime() {
    auto now = std::chrono::system_clock::now();
    auto time = std::chrono::system_clock::to_time_t(now);
    auto ms = std::chrono::duration_cast<std::chrono::milliseconds>(
        now.time_since_epoch()) % 1000;
    
    std::ostringstream oss;
    oss << std::put_time(std::localtime(&time), "%Y-%m-%d %H:%M:%S");
    oss << '.' << std::setfill('0') << std::setw(3) << ms.count();
    return oss.str();
}

std::string Logger::levelToString(Level level) {
    switch (level) {
        case Level::DEBUG:    return "DEBUG";
        case Level::INFO:     return "INFO";
        case Level::WARNING:  return "WARNING";
        case Level::ERROR:    return "ERROR";
        case Level::CRITICAL: return "CRITICAL";
        default:              return "UNKNOWN";
    }
}

void Logger::writeToLog(const std::string& message, Level level) {
    std::lock_guard<std::mutex> lock(logMutex);
    
    std::ostringstream logEntry;
    logEntry << "[" << getCurrentTime() << "] "
             << "[" << levelToString(level) << "] "
             << message;
    
    std::string formattedMessage = logEntry.str();
    
    if (consoleOutputEnabled) {
        // Color coding for console output
        switch (level) {
            case Level::ERROR:
            case Level::CRITICAL:
                std::cerr << "\033[1;31m" << formattedMessage << "\033[0m" << std::endl;
                break;
            case Level::WARNING:
                std::cout << "\033[1;33m" << formattedMessage << "\033[0m" << std::endl;
                break;
            case Level::INFO:
                std::cout << "\033[1;32m" << formattedMessage << "\033[0m" << std::endl;
                break;
            default:
                std::cout << formattedMessage << std::endl;
                break;
        }
    }
    
    if (fileOutputEnabled && logFileStream.is_open()) {
        logFileStream << formattedMessage << std::endl;
        logFileStream.flush();
    }
}
"#;

#[cfg(test)]
const VALIDATION_HPP: &str = r#"#ifndef VALIDATION_HPP
#define VALIDATION_HPP

#include <string>
#include <regex>
#include <vector>
#include <algorithm>

class Validation {
public:
    // Email validation
    static bool isValidEmail(const std::string& email);
    
    // Phone number validation (US format)
    static bool isValidPhone(const std::string& phone);
    
    // SSN validation
    static bool isValidSSN(const std::string& ssn);
    
    // Credit card validation (basic Luhn check)
    static bool isValidCreditCard(const std::string& cardNumber);
    
    // Password validation
    static bool isValidPassword(const std::string& password);
    
    // Date validation (YYYY-MM-DD)
    static bool isValidDate(const std::string& date);
    
    // URL validation
    static bool isValidURL(const std::string& url);
    
    // IP address validation
    static bool isValidIPv4(const std::string& ip);
    static bool isValidIPv6(const std::string& ip);
    
    // Numeric range validation
    template<typename T>
    static bool isInRange(T value, T min, T max);
    
    // String length validation
    static bool isValidLength(const std::string& str, 
                              size_t minLength, 
                              size_t maxLength);
    
    // Character set validation
    static bool containsOnly(const std::string& str, 
                             const std::string& allowedChars);
    static bool containsNone(const std::string& str, 
                             const std::string& forbiddenChars);
    
    // File extension validation
    static bool hasValidExtension(const std::string& filename, 
                                  const std::vector<std::string>& allowedExtensions);
    
    // Business rule validations
    static bool isValidAge(int age);
    static bool isValidZipCode(const std::string& zipCode);
    static bool isValidStateCode(const std::string& stateCode);
    
    // Custom validation with predicate
    template<typename T, typename Predicate>
    static bool validateWithPredicate(const T& value, Predicate pred);
};

// Template implementations
template<typename T>
bool Validation::isInRange(T value, T min, T max) {
    return value >= min && value <= max;
}

template<typename T, typename Predicate>
bool Validation::validateWithPredicate(const T& value, Predicate pred) {
    return pred(value);
}

#endif // VALIDATION_HPP
"#;

#[cfg(test)]
const VALIDATION_CPP: &str = r#"#include "validation.hpp"
#include <cctype>
#include <sstream>
#include <iomanip>

bool Validation::isValidEmail(const std::string& email) {
    // RFC 5322 compliant regex (simplified version)
    const std::regex pattern(
        R"(^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$)"
    );
    return std::regex_match(email, pattern);
}

bool Validation::isValidPhone(const std::string& phone) {
    // Remove all non-digit characters
    std::string digits;
    std::copy_if(phone.begin(), phone.end(), std::back_inserter(digits),
                 [](char c) { return std::isdigit(c); });
    
    // Check if it's 10 digits (US format)
    if (digits.length() != 10) {
        return false;
    }
    
    // Additional validation: area code cannot start with 0 or 1
    if (digits[0] == '0' || digits[0] == '1') {
        return false;
    }
    
    return true;
}

bool Validation::isValidSSN(const std::string& ssn) {
    // Remove all non-digit characters
    std::string digits;
    std::copy_if(ssn.begin(), ssn.end(), std::back_inserter(digits),
                 [](char c) { return std::isdigit(c); });
    
    // Must be exactly 9 digits
    if (digits.length() != 9) {
        return false;
    }
    
    // SSN cannot start with 000, 666, or 900-999
    std::string area = digits.substr(0, 3);
    int areaNum = std::stoi(area);
    
    if (area == "000" || area == "666" || (areaNum >= 900 && areaNum <= 999)) {
        return false;
    }
    
    // Group number (middle two digits) cannot be 00
    if (digits.substr(3, 2) == "00") {
        return false;
    }
    
    // Serial number (last four digits) cannot be 0000
    if (digits.substr(5, 4) == "0000") {
        return false;
    }
    
    return true;
}

bool Validation::isValidCreditCard(const std::string& cardNumber) {
    // Remove all non-digit characters
    std::string digits;
    std::copy_if(cardNumber.begin(), cardNumber.end(), std::back_inserter(digits),
                 [](char c) { return std::isdigit(c); });
    
    // Check length (typically 13-19 digits)
    if (digits.length() < 13 || digits.length() > 19) {
        return false;
    }
    
    // Luhn algorithm
    int sum = 0;
    bool alternate = false;
    
    for (int i = digits.length() - 1; i >= 0; i--) {
        int n = digits[i] - '0';
        
        if (alternate) {
            n *= 2;
            if (n > 9) {
                n = (n % 10) + 1;
            }
        }
        
        sum += n;
        alternate = !alternate;
    }
    
    return (sum % 10 == 0);
}

bool Validation::isValidPassword(const std::string& password) {
    // At least 8 characters
    if (password.length() < 8) {
        return false;
    }
    
    // Check for at least one uppercase letter
    bool hasUpper = std::any_of(password.begin(), password.end(),
                                [](char c) { return std::isupper(c); });
    if (!hasUpper) {
        return false;
    }
    
    // Check for at least one lowercase letter
    bool hasLower = std::any_of(password.begin(), password.end(),
                                [](char c) { return std::islower(c); });
    if (!hasLower) {
        return false;
    }
    
    // Check for at least one digit
    bool hasDigit = std::any_of(password.begin(), password.end(),
                                [](char c) { return std::isdigit(c); });
    if (!hasDigit) {
        return false;
    }
    
    // Check for at least one special character
    bool hasSpecial = std::any_of(password.begin(), password.end(),
                                  [](char c) { return !std::isalnum(c); });
    if (!hasSpecial) {
        return false;
    }
    
    return true;
}

bool Validation::isValidDate(const std::string& date) {
    const std::regex pattern(
        R"(^\d{4}-(0[1-9]|1[0-2])-(0[1-9]|[12]\d|3[01])$)"
    );
    
    if (!std::regex_match(date, pattern)) {
        return false;
    }
    
    // Additional validation for days in month
    int year = std::stoi(date.substr(0, 4));
    int month = std::stoi(date.substr(5, 2));
    int day = std::stoi(date.substr(8, 2));
    
    // Check for valid day in month
    if (month == 2) {
        // February
        bool isLeapYear = (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0);
        if (day > (isLeapYear ? 29 : 28)) {
            return false;
        }
    } else if (month == 4 || month == 6 || month == 9 || month == 11) {
        // Months with 30 days
        if (day > 30) {
            return false;
        }
    }
    
    return true;
}

bool Validation::isValidURL(const std::string& url) {
    const std::regex pattern(
        R"(^(https?|ftp)://[^\s/$.?#].[^\s]*$)"
    );
    return std::regex_match(url, pattern);
}

bool Validation::isValidIPv4(const std::string& ip) {
    const std::regex pattern(
        R"(^(\d{1,3})\.(\d{1,3})\.(\d{1,3})\.(\d{1,3})$)"
    );
    
    std::smatch matches;
    if (!std::regex_match(ip, matches, pattern)) {
        return false;
    }
    
    // Check each octet is between 0 and 255
    for (int i = 1; i <= 4; i++) {
        int octet = std::stoi(matches[i]);
        if (octet < 0 || octet > 255) {
            return false;
        }
    }
    
    return true;
}

bool Validation::isValidIPv6(const std::string& ip) {
    const std::regex pattern(
        R"(^([0-9a-fA-F]{1,4}:){7}[0-9a-fA-F]{1,4}$)"
    );
    return std::regex_match(ip, pattern);
}

bool Validation::isValidLength(const std::string& str, 
                               size_t minLength, 
                               size_t maxLength) {
    size_t length = str.length();
    return length >= minLength && length <= maxLength;
}

bool Validation::containsOnly(const std::string& str, 
                              const std::string& allowedChars) {
    return std::all_of(str.begin(), str.end(),
                       [&allowedChars](char c) {
                           return allowedChars.find(c) != std::string::npos;
                       });
}

bool Validation::containsNone(const std::string& str, 
                              const std::string& forbiddenChars) {
    return std::none_of(str.begin(), str.end(),
                        [&forbiddenChars](char c) {
                            return forbiddenChars.find(c) != std::string::npos;
                        });
}

bool Validation::hasValidExtension(const std::string& filename, 
                                   const std::vector<std::string>& allowedExtensions) {
    size_t dotPos = filename.find_last_of('.');
    if (dotPos == std::string::npos) {
        return false;
    }
    
    std::string extension = filename.substr(dotPos + 1);
    std::transform(extension.begin(), extension.end(), extension.begin(),
                   [](unsigned char c) { return std::tolower(c); });
    
    return std::find(allowedExtensions.begin(), allowedExtensions.end(), 
                     extension) != allowedExtensions.end();
}

bool Validation::isValidAge(int age) {
    return age >= 0 && age <= 150;
}

bool Validation::isValidZipCode(const std::string& zipCode) {
    // US ZIP code: 5 digits or 5+4 format
    const std::regex pattern(R"(^\d{5}(-\d{4})?$)");
    return std::regex_match(zipCode, pattern);
}

bool Validation::isValidStateCode(const std::string& stateCode) {
    static const std::vector<std::string> validStates = {
        "AL", "AK", "AZ", "AR", "CA", "CO", "CT", "DE", "FL", "GA",
        "HI", "ID", "IL", "IN", "IA", "KS", "KY", "LA", "ME", "MD",
        "MA", "MI", "MN", "MS", "MO", "MT", "NE", "NV", "NH", "NJ",
        "NM", "NY", "NC", "ND", "OH", "OK", "OR", "PA", "RI", "SC",
        "SD", "TN", "TX", "UT", "VT", "VA", "WA", "WV", "WI", "WY"
    };
    
    if (stateCode.length() != 2) {
        return false;
    }
    
    std::string upperCode = stateCode;
    std::transform(upperCode.begin(), upperCode.end(), upperCode.begin(),
                   [](unsigned char c) { return std::toupper(c); });
    
    return std::find(validStates.begin(), validStates.end(), upperCode) != validStates.end();
}
"#;

#[cfg(test)]
const CPPMAIN_TEMPLATE: &str = r#"#include <iostream>
#include <vector>
#include "calculator.hpp"
#include "string_utils.hpp"
#include "data_processor.hpp"
#include "logger.hpp"
#include "validation.hpp"

int main() {
    // Initialize logger
    Logger::init();
    Logger::info("Starting comprehensive test application");
    
    // ========== Test Calculator ==========
    Logger::info("Testing Calculator module");
    std::cout << "=== Calculator Tests ===" << std::endl;
    
    // Basic arithmetic
    std::cout << "5 + 3 = " << Calculator::add(5, 3) << std::endl;
    std::cout << "10 - 4 = " << Calculator::subtract(10, 4) << std::endl;
    std::cout << "6 * 7 = " << Calculator::multiply(6, 7) << std::endl;
    std::cout << "15 / 3 = " << Calculator::divide(15, 3) << std::endl;
    
    // Advanced math
    std::cout << "Factorial of 5 = " << Calculator::factorial(5) << std::endl;
    std::cout << "Is 17 prime? " << (Calculator::isPrime(17) ? "Yes" : "No") << std::endl;
    std::cout << "GCD of 48 and 18 = " << Calculator::gcd(48, 18) << std::endl;
    std::cout << "LCM of 12 and 15 = " << Calculator::lcm(12, 15) << std::endl;
    
    // Statistics
    std::vector<int> numbers = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    std::cout << "Mean of numbers = " << Calculator::mean(numbers) << std::endl;
    std::cout << "Standard deviation = " << Calculator::standardDeviation(numbers) << std::endl;
    
    // ========== Test StringUtils ==========
    Logger::info("Testing StringUtils module");
    std::cout << "\n=== StringUtils Tests ===" << std::endl;
    
    std::string testString = "  Hello, World!  ";
    std::cout << "Original: '" << testString << "'" << std::endl;
    std::cout << "Trimmed: '" << StringUtils::trim(testString) << "'" << std::endl;
    std::cout << "Uppercase: '" << StringUtils::toUpper(testString) << "'" << std::endl;
    std::cout << "Reversed: '" << StringUtils::reverse("Hello") << "'" << std::endl;
    
    // String checks
    std::cout << "Starts with 'Hello'? " 
              << (StringUtils::startsWith("Hello World", "Hello") ? "Yes" : "No") << std::endl;
    std::cout << "Contains 'World'? " 
              << (StringUtils::contains("Hello World", "World") ? "Yes" : "No") << std::endl;
    
    // Case conversion
    std::cout << "Camel case: '" << StringUtils::toCamelCase("hello_world_test") << "'" << std::endl;
    std::cout << "Snake case: '" << StringUtils::toSnakeCase("HelloWorldTest") << "'" << std::endl;
    
    // ========== Test DataProcessor ==========
    Logger::info("Testing DataProcessor module");
    std::cout << "\n=== DataProcessor Tests ===" << std::endl;
    
    DataProcessor processor;
    std::vector<int> data = {5, 3, 8, 1, 9, 2, 7, 4, 6, 10};
    
    std::cout << "Original data: ";
    for (int n : data) std::cout << n << " ";
    std::cout << std::endl;
    
    std::cout << "Sum: " << processor.sum(data) << std::endl;
    std::cout << "Average: " << processor.average(data) << std::endl;
    std::cout << "Max: " << processor.max(data) << std::endl;
    std::cout << "Min: " << processor.min(data) << std::endl;
    std::cout << "Median: " << processor.median(data) << std::endl;
    
    auto evenNumbers = processor.filterEven(data);
    std::cout << "Even numbers: ";
    for (int n : evenNumbers) std::cout << n << " ";
    std::cout << std::endl;
    
    auto squared = processor.square(data);
    std::cout << "Squared values: ";
    for (int n : squared) std::cout << n << " ";
    std::cout << std::endl;
    
    // ========== Test Validation ==========
    Logger::info("Testing Validation module");
    std::cout << "\n=== Validation Tests ===" << std::endl;
    
    // Email validation
    std::cout << "Email 'test@example.com' valid? " 
              << (Validation::isValidEmail("test@example.com") ? "Yes" : "No") << std::endl;
    std::cout << "Email 'invalid-email' valid? " 
              << (Validation::isValidEmail("invalid-email") ? "Yes" : "No") << std::endl;
    
    // Phone validation
    std::cout << "Phone '1234567890' valid? " 
              << (Validation::isValidPhone("1234567890") ? "Yes" : "No") << std::endl;
    std::cout << "Phone '0123456789' valid? " 
              << (Validation::isValidPhone("0123456789") ? "Yes" : "No") << std::endl;
    
    // Date validation
    std::cout << "Date '2024-12-25' valid? " 
              << (Validation::isValidDate("2024-12-25") ? "Yes" : "No") << std::endl;
    std::cout << "Date '2024-02-30' valid? " 
              << (Validation::isValidDate("2024-02-30") ? "Yes" : "No") << std::endl;
    
    // Password validation
    std::cout << "Password 'Pass123!' valid? " 
              << (Validation::isValidPassword("Pass123!") ? "Yes" : "No") << std::endl;
    std::cout << "Password 'weak' valid? " 
              << (Validation::isValidPassword("weak") ? "Yes" : "No") << std::endl;
    
    // ========== Integration Test ==========
    Logger::info("Running integration tests");
    std::cout << "\n=== Integration Test ===" << std::endl;
    
    // Process data and validate
    std::vector<int> testData = {10, 20, 30, 40, 50};
    DataProcessor dp;
    
    // Filter and process
    auto filtered = dp.filterEven(testData);
    auto processed = dp.process(filtered, [](int x) { return x / 2; });
    
    std::cout << "Processed data: ";
    for (int n : processed) std::cout << n << " ";
    std::cout << std::endl;
    
    // Validate the results
    bool allValid = true;
    for (int n : processed) {
        if (!Validation::isInRange(n, 5, 25)) {
            allValid = false;
            Logger::warning("Value " + std::to_string(n) + " is out of range");
        }
    }
    
    if (allValid) {
        Logger::info("All processed values are within valid range");
        std::cout << "All values are valid!" << std::endl;
    }
    
    // ========== Cleanup ==========
    Logger::info("Application completed successfully");
    Logger::cleanup();
    
    std::cout << "\n=== Application Complete ===" << std::endl;
    std::cout << "All tests passed. Check app.log for detailed logs." << std::endl;
    
    return 0;
}
"#;

// ===================== C TEMPLATES =====================

#[cfg(test)]
const CTEMPLATE_MAIN: &str = r#"#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include "calculator.h"
#include "string_utils.h"
#include "data_processor.h"
#include "logger.h"
#include "validation.h"

int main() {
    printf("=== C Test Application ===\n\n");
    
    // Test Calculator
    printf("Calculator Tests:\n");
    printf("5 + 3 = %d\n", add(5, 3));
    printf("10 * 2 = %d\n", multiply(10, 2));
    printf("Factorial of 5 = %d\n", factorial(5));
    printf("Is 17 prime? %s\n", is_prime(17) ? "Yes" : "No");
    
    // Test StringUtils
    printf("\nStringUtils Tests:\n");
    char test_str[] = "Hello, World!";
    printf("Original: %s\n", test_str);
    
    char* reversed = reverse_string(test_str);
    printf("Reversed: %s\n", reversed);
    free(reversed);
    
    char* upper = to_upper(test_str);
    printf("Uppercase: %s\n", upper);
    free(upper);
    
    // Test DataProcessor
    printf("\nDataProcessor Tests:\n");
    int data[] = {1, 2, 3, 4, 5, 6, 7, 8, 9, 10};
    int data_size = 10;
    
    printf("Sum: %.2f\n", calculate_sum(data, data_size));
    printf("Average: %.2f\n", calculate_average(data, data_size));
    printf("Max: %d\n", find_max(data, data_size));
    
    // Test Validation
    printf("\nValidation Tests:\n");
    printf("Is 'test@email.com' valid email? %s\n", 
           is_valid_email("test@email.com") ? "Yes" : "No");
    printf("Is '123-45-6789' valid SSN? %s\n",
           is_valid_ssn("123-45-6789") ? "Yes" : "No");
    
    // Test Logger
    printf("\nLogger Tests:\n");
    logger_init("c_app.log");
    log_message("INFO", "Application started");
    log_message("WARNING", "This is a warning");
    log_message("ERROR", "This is an error");
    logger_cleanup();
    printf("Check c_app.log for logs\n");
    
    printf("\n=== All Tests Completed ===\n");
    return 0;
}
"#;

// ... (C templates for calculator.h, calculator.c, etc. would follow similar pattern)
// For brevity, I'm showing the structure but the C templates would be similar to C++
// but written in C syntax without classes

// ===================== TEMPLATE COLLECTIONS =====================

#[cfg(test)]
const CPPTEMPLATES: &[(&str, &str)] = &[
    ("include/calculator.hpp", CALCULATOR_HPP),
    ("src/calculator.cpp", CALCULATOR_CPP),
    ("include/string_utils.hpp", STRING_UTILS_HPP),
    ("src/string_utils.cpp", STRING_UTILS_CPP),
    ("include/data_processor.hpp", DATA_PROCESSOR_HPP),
    ("src/data_processor.cpp", DATA_PROCESSOR_CPP),
    ("include/logger.hpp", LOGGER_HPP),
    ("src/logger.cpp", LOGGER_CPP),
    ("include/validation.hpp", VALIDATION_HPP),
    ("src/validation.cpp", VALIDATION_CPP),
];

#[cfg(test)]
const CTEMPLATES: &[(&str, &str)] = &[
    ("include/calculator.h", CALCULATOR_H),
    ("src/calculator.c", CALCULATOR_C),
    ("include/string_utils.h", STRING_UTILS_H),
    ("src/string_utils.c", STRING_UTILS_C),
    ("include/data_processor.h", DATA_PROCESSOR_H),
    ("src/data_processor.c", DATA_PROCESSOR_C),
    ("include/logger.h", LOGGER_H),
    ("src/logger.c", LOGGER_C),
    ("include/validation.h", VALIDATION_H),
    ("src/validation.c", VALIDATION_C),
];

// ===================== C HEADER/SOURCE TEMPLATES =====================

#[cfg(test)]
const CALCULATOR_H: &str = r#"#ifndef CALCULATOR_H
#define CALCULATOR_H

int add(int a, int b);
int subtract(int a, int b);
int multiply(int a, int b);
double divide(int a, int b);
int factorial(int n);
int is_prime(int n);
int gcd(int a, int b);

#endif // CALCULATOR_H
"#;

#[cfg(test)]
const CALCULATOR_C: &str = r#"#include "calculator.h"

int add(int a, int b) {
    return a + b;
}

int subtract(int a, int b) {
    return a - b;
}

int multiply(int a, int b) {
    return a * b;
}

double divide(int a, int b) {
    if (b == 0) return 0;
    return (double)a / b;
}

int factorial(int n) {
    if (n <= 1) return 1;
    int result = 1;
    for (int i = 2; i <= n; i++) {
        result *= i;
    }
    return result;
}

int is_prime(int n) {
    if (n <= 1) return 0;
    if (n <= 3) return 1;
    if (n % 2 == 0 || n % 3 == 0) return 0;
    for (int i = 5; i * i <= n; i += 6) {
        if (n % i == 0 || n % (i + 2) == 0) return 0;
    }
    return 1;
}

int gcd(int a, int b) {
    while (b != 0) {
        int temp = b;
        b = a % b;
        a = temp;
    }
    return a;
}
"#;

#[cfg(test)]
const STRING_UTILS_H: &str = r#"#ifndef STRING_UTILS_H
#define STRING_UTILS_H

char* reverse_string(const char* str);
char* to_upper(const char* str);
char* to_lower(const char* str);
char* trim(const char* str);
int starts_with(const char* str, const char* prefix);
int contains(const char* str, const char* substr);

#endif // STRING_UTILS_H
"#;

#[cfg(test)]
const STRING_UTILS_C: &str = r#"#include "string_utils.h"
#include <stdlib.h>
#include <string.h>
#include <ctype.h>

char* reverse_string(const char* str) {
    size_t len = strlen(str);
    char* result = (char*)malloc(len + 1);
    for (size_t i = 0; i < len; i++) {
        result[i] = str[len - 1 - i];
    }
    result[len] = '\0';
    return result;
}

char* to_upper(const char* str) {
    size_t len = strlen(str);
    char* result = (char*)malloc(len + 1);
    for (size_t i = 0; i < len; i++) {
        result[i] = toupper((unsigned char)str[i]);
    }
    result[len] = '\0';
    return result;
}

char* to_lower(const char* str) {
    size_t len = strlen(str);
    char* result = (char*)malloc(len + 1);
    for (size_t i = 0; i < len; i++) {
        result[i] = tolower((unsigned char)str[i]);
    }
    result[len] = '\0';
    return result;
}

char* trim(const char* str) {
    while (isspace((unsigned char)*str)) str++;
    if (*str == '\0') {
        char* result = (char*)malloc(1);
        result[0] = '\0';
        return result;
    }
    const char* end = str + strlen(str) - 1;
    while (end > str && isspace((unsigned char)*end)) end--;
    size_t len = end - str + 1;
    char* result = (char*)malloc(len + 1);
    memcpy(result, str, len);
    result[len] = '\0';
    return result;
}

int starts_with(const char* str, const char* prefix) {
    return strncmp(str, prefix, strlen(prefix)) == 0;
}

int contains(const char* str, const char* substr) {
    return strstr(str, substr) != NULL;
}
"#;

#[cfg(test)]
const DATA_PROCESSOR_H: &str = r#"#ifndef DATA_PROCESSOR_H
#define DATA_PROCESSOR_H

double calculate_sum(const int* data, int size);
double calculate_average(const int* data, int size);
int find_max(const int* data, int size);
int find_min(const int* data, int size);

#endif // DATA_PROCESSOR_H
"#;

#[cfg(test)]
const DATA_PROCESSOR_C: &str = r#"#include "data_processor.h"

double calculate_sum(const int* data, int size) {
    double sum = 0;
    for (int i = 0; i < size; i++) {
        sum += data[i];
    }
    return sum;
}

double calculate_average(const int* data, int size) {
    if (size == 0) return 0;
    return calculate_sum(data, size) / size;
}

int find_max(const int* data, int size) {
    if (size == 0) return 0;
    int max = data[0];
    for (int i = 1; i < size; i++) {
        if (data[i] > max) max = data[i];
    }
    return max;
}

int find_min(const int* data, int size) {
    if (size == 0) return 0;
    int min = data[0];
    for (int i = 1; i < size; i++) {
        if (data[i] < min) min = data[i];
    }
    return min;
}
"#;

#[cfg(test)]
const LOGGER_H: &str = r#"#ifndef LOGGER_H
#define LOGGER_H

void logger_init(const char* filename);
void logger_cleanup(void);
void log_message(const char* level, const char* message);

#endif // LOGGER_H
"#;

#[cfg(test)]
const LOGGER_C: &str = r#"#include "logger.h"
#include <stdio.h>
#include <time.h>

static FILE* log_file = NULL;

void logger_init(const char* filename) {
    log_file = fopen(filename, "a");
}

void logger_cleanup(void) {
    if (log_file) {
        fclose(log_file);
        log_file = NULL;
    }
}

void log_message(const char* level, const char* message) {
    time_t now = time(NULL);
    char* time_str = ctime(&now);
    time_str[24] = '\0'; // Remove newline
    
    printf("[%s] [%s] %s\n", time_str, level, message);
    if (log_file) {
        fprintf(log_file, "[%s] [%s] %s\n", time_str, level, message);
        fflush(log_file);
    }
}
"#;

#[cfg(test)]
const VALIDATION_H: &str = r#"#ifndef VALIDATION_H
#define VALIDATION_H

int is_valid_email(const char* email);
int is_valid_ssn(const char* ssn);
int is_valid_phone(const char* phone);

#endif // VALIDATION_H
"#;

#[cfg(test)]
const VALIDATION_C: &str = r#"#include "validation.h"
#include <string.h>
#include <ctype.h>

int is_valid_email(const char* email) {
    const char* at = strchr(email, '@');
    if (!at || at == email) return 0;
    const char* dot = strchr(at, '.');
    if (!dot || dot == at + 1 || *(dot + 1) == '\0') return 0;
    return 1;
}

int is_valid_ssn(const char* ssn) {
    int digits = 0;
    for (const char* p = ssn; *p; p++) {
        if (isdigit((unsigned char)*p)) digits++;
        else if (*p != '-') return 0;
    }
    return digits == 9;
}

int is_valid_phone(const char* phone) {
    int digits = 0;
    for (const char* p = phone; *p; p++) {
        if (isdigit((unsigned char)*p)) digits++;
    }
    return digits == 10;
}
"#;

// ===================== TEST PROJECT FIXTURE TEMPLATES =====================

#[cfg(test)]
const FIXTURE_MAIN_CPP: &str = r#"#include <iostream>
#include "logger.h"
#include "math_utils.h"
#include "utils.h"

int main() {
    log_info("Application started");
    
    int result = add(5, 3);
    std::cout << "5 + 3 = " << result << std::endl;
    
    int product = multiply(4, 7);
    std::cout << "4 * 7 = " << product << std::endl;
    
    std::cout << "Value: " << get_value() << std::endl;
    
    log_info("Application finished");
    return 0;
}
"#;

#[cfg(test)]
const FIXTURE_LOGGER_H: &str = r#"#ifndef LOGGER_H
#define LOGGER_H

void log_info(const char* msg);
void log_error(const char* msg);
void log_debug(const char* msg);

#endif // LOGGER_H
"#;

#[cfg(test)]
const FIXTURE_LOGGER_CPP: &str = r#"#include "logger.h"
#include <iostream>

void log_info(const char* msg) {
    std::cout << "[INFO] " << msg << std::endl;
}

void log_error(const char* msg) {
    std::cerr << "[ERROR] " << msg << std::endl;
}

void log_debug(const char* msg) {
    std::cout << "[DEBUG] " << msg << std::endl;
}
"#;

#[cfg(test)]
const FIXTURE_MATH_UTILS_H: &str = r#"#ifndef MATH_UTILS_H
#define MATH_UTILS_H

int add(int a, int b);
int subtract(int a, int b);
int multiply(int a, int b);
int divide(int a, int b);

#endif // MATH_UTILS_H
"#;

#[cfg(test)]
const FIXTURE_MATH_UTILS_CPP: &str = r#"#include "math_utils.h"

int add(int a, int b) {
    return a + b;
}

int subtract(int a, int b) {
    return a - b;
}

int multiply(int a, int b) {
    return a * b;
}

int divide(int a, int b) {
    if (b == 0) return 0;
    return a / b;
}
"#;

#[cfg(test)]
const FIXTURE_UTILS_H: &str = r#"#ifndef UTILS_H
#define UTILS_H

int get_value();
void print_hello();

#endif // UTILS_H
"#;

#[cfg(test)]
const FIXTURE_UTILS_CPP: &str = r#"#include "utils.h"
#include <iostream>

int get_value() {
    return 42;
}

void print_hello() {
    std::cout << "Hello from utils!" << std::endl;
}
"#;


#[cfg(test)]
mod tests {
    use crate::test_util::fstree::FsTree;

    use std::path::Path;


    #[test]
    fn test_fs_tree() {

        let mut fs_t = FsTree::new(Path::new("test_project")).expect("Failed to create FsTree");
        fs_t.create_minimal_project(true).expect("Failed to create structure");
        assert!(fs_t.root().exists());

       _ = fs_t.create_test_project_fixture().expect("Failed to create fixture");
       _= fs_t.create_full_project(false) ;

        let resp = fs_t.create_minimal_project(true) ;
        assert_ne!(resp, Ok(()), "Should not recreate existing structure");
        
        // Verify structure
        assert!(fs_t.root().join("src").exists());
        assert!(fs_t.root().join("include").exists());
        assert!(fs_t.root().join("build").exists());
        assert!(fs_t.root().join("src/main.cpp").exists());
        assert!(fs_t.root().join("include/logger.h").exists());
        
        
    }
}